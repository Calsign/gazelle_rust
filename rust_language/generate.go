package rust_language

import (
	"log"
	"os"
	"path"
	"path/filepath"
	"strings"

	"github.com/bazelbuild/bazel-gazelle/config"
	"github.com/bazelbuild/bazel-gazelle/label"
	"github.com/bazelbuild/bazel-gazelle/language"
	"github.com/bazelbuild/bazel-gazelle/rule"

	pb "github.com/calsign/gazelle_rust/proto"
)

func (l *rustLang) isTestDir(dirname *string) bool {
	return dirname != nil && (*dirname == "test" || *dirname == "tests")
}

func (l *rustLang) isTestFilename(filename string) bool {
	return strings.HasSuffix(filename, "_test.rs") || strings.HasPrefix(filename, "test_")
}

// Infer the default kind for a new target (e.g. rust_library, rust_binary).
func (l *rustLang) inferRuleKind(filename string, dirname *string,
	response *pb.RustImportsResponse) string {

	if response.Hints.HasProcMacro {
		// only proc-macro crates are allowed to have #[proc_macro] functions
		return "rust_proc_macro"
	} else if response.Hints.HasMain {
		// while not necessarily true, having a top-level main function is a strong
		// indicator that this is a binary
		return "rust_binary"
	} else if filename == "main.rs" {
		return "rust_binary"
	} else if filename == "lib.rs" {
		return "rust_library"
	} else if response.Hints.HasTest && (l.isTestDir(dirname) || l.isTestFilename(filename)) {
		// assume that sources with tests in a test/tests directory are integration tests
		// assume that sources with tests with test-like names are integration tests
		return "rust_test"
	} else {
		return "rust_library"
	}
}

type RuleData struct {
	rule      *rule.Rule
	responses []*pb.RustImportsResponse
	// if a test crate referring to another crate, that crate; otherwise, nil
	testedCrate *rule.Rule
	// the build script of this crate, if any
	buildScript *label.Label
	// name of the parent crate
	parentCrateName string
}

func getTestCrate(rule *rule.Rule, repo string, pkg string) string {
	crateName := rule.AttrString("crate")
	if crateName != "" {
		label, err := label.Parse(crateName)
		if err == nil {
			rel := label.Rel(repo, pkg)
			if rel.Relative {
				return label.Name
			}
		}
	}
	return ""
}

// If there is already a rule with the requested name, we want to be able to fall back to a fresh
// name, by adding an "_rs" suffix. It's possible (although unlikely) that a rule with that suffixed
// name also exists, in which case we fail and return nil.
func freshRuleName(request string, existingRuleNames map[string]bool) *string {
	if _, ok := existingRuleNames[request]; ok {
		// need to pick a new name
		suffixedName := request + "_rs"
		if _, ok := existingRuleNames[suffixedName]; ok {
			// give up
			return nil
		} else {
			return &suffixedName
		}
	} else {
		// we can use the request
		return &request
	}
}

var ruleCloneAttrs = []string{"srcs", "crate", "crate_root"}

// It's nice to be able to re-use existing Rules so that we can resolve them but preserve the
// grouping of srcs, which is not something Gazelle handles natively. By making a new rule with the
// attrs that we want to preserve (e.g., srcs), we preserve the existing groupings. If we were to
// reuse the existing rule without cloning it, certain things like #keep comments stop working.
func CloneRule(oldRule *rule.Rule) *rule.Rule {
	newRule := rule.NewRule(oldRule.Kind(), oldRule.Name())
	for _, attr := range ruleCloneAttrs {
		if val := oldRule.Attr(attr); val != nil {
			newRule.SetAttr(attr, val)
		}
	}
	return newRule
}

func (l *rustLang) GenerateRules(args language.GenerateArgs) language.GenerateResult {
	cfg := l.GetConfig(args.Config)
	switch cfg.Mode {
	case modePureBazel:
		return l.generateRulesPureBazel(args)
	case modeGenerateFromCargo:
		return l.generateRulesFromCargo(args)
	default:
		log.Panicf("unrecognized mode")
		return language.GenerateResult{}
	}
}

func (l *rustLang) generateRulesPureBazel(args language.GenerateArgs) language.GenerateResult {
	result := language.GenerateResult{}

	filesInExistingRules := map[string]bool{}
	existingRuleNames := map[string]bool{}

	var dirname *string
	if args.Rel == "" {
		dirname = nil
	} else {
		base := path.Base(args.Rel)
		dirname = &base
	}

	// list of all non-rust_test rules; these may generate additional crate test targets
	nonTestRules := []RuleData{}
	// map of crate test rules; key is the non-rust_test rule name that each one refers to
	testRules := make(map[string]*rule.Rule)

	// Discover modules claimed by lib.rs crate roots before processing new files.
	// This prevents creating standalone rules for files that are modules of a lib.rs crate.
	filesInCrates := map[string]bool{}

	// First, check if any files in this directory are claimed by a parent lib.rs crate.
	// This handles subdirectory modules (e.g., subdir/mod.rs claimed by parent's lib.rs).
	for file := range l.getFilesClaimedByParent(args.Config, &args) {
		filesInCrates[file] = true
	}

	// Then discover modules claimed by lib.rs crate roots in THIS directory.
	for _, file := range args.RegularFiles {
		if strings.HasSuffix(file, ".rs") && filepath.Base(file) == "lib.rs" {
			importsResponses := map[string]*pb.RustImportsResponse{}
			l.discoverModule(args.Config, file, &args, &importsResponses, true)
			for source := range importsResponses {
				filesInCrates[source] = true
			}
		}
	}

	addRule := func(rule *rule.Rule, responses []*pb.RustImportsResponse) {
		ruleData := RuleData{
			rule:        rule,
			responses:   responses,
			testedCrate: nil,
		}

		result.Gen = append(result.Gen, rule)
		result.Imports = append(result.Imports, ruleData)

		if rule.Kind() == "rust_test" {
			if crateName := getTestCrate(rule, args.Config.RepoName, args.Rel); crateName != "" {
				if _, ok := testRules[crateName]; ok {
					l.Log(args.Config, logWarn, args.File, "found multiple crate test rules for %s\n", crateName)
				}
				testRules[crateName] = rule
			}
		} else {
			nonTestRules = append(nonTestRules, ruleData)
		}
	}

	if args.File != nil {
		for _, existingRule := range args.File.Rules {
			existingRuleNames[existingRule.Name()] = true

			unmappedKind := l.GetMappedKindInverse(args.Config, existingRule.Kind())

			if SliceContains(commonDefs, unmappedKind) {
				rule := CloneRule(existingRule)

				// NOTE: Gazelle expects us to create rules using the un-mapped kinds. Since we are
				// re-creating an existing rule, the associated kind is the mapped one, and we need to
				// reset it. It is probably a bug that Gazelle does not already handle this for us.
				rule.SetKind(unmappedKind)

				// Check if this rule is a lib.rs crate
				crateRoot := rule.AttrString("crate_root")
				if crateRoot == "" {
					srcs := rule.AttrStrings("srcs")
					for _, src := range srcs {
						if filepath.Base(src) == "lib.rs" {
							crateRoot = src
							break
						}
					}
				}
				isLibCrate := filepath.Base(crateRoot) == "lib.rs"

				responses := []*pb.RustImportsResponse{}
				validSrcs := []string{}

				if isLibCrate && crateRoot != "" {
					// For lib.rs crates, re-discover all modules
					importsResponses := map[string]*pb.RustImportsResponse{}
					l.discoverModule(args.Config, crateRoot, &args, &importsResponses, true)
					for source, sourceResponse := range importsResponses {
						filesInExistingRules[source] = true
						validSrcs = append(validSrcs, source)
						if sourceResponse != nil {
							responses = append(responses, sourceResponse)
						}
					}
				} else {
					for _, file := range rule.AttrStrings("srcs") {
						// Skip files that no longer exist on disk
						filePath := path.Join(args.Dir, file)
						if _, err := os.Stat(filePath); os.IsNotExist(err) {
							continue
						}

						filesInExistingRules[file] = true
						validSrcs = append(validSrcs, file)

						if strings.HasSuffix(file, ".rs") {
							response := l.parseFile(args.Config, file, &args)
							if response != nil {
								responses = append(responses, response)
							}
						}
					}
				}

				if len(validSrcs) > 0 {
					rule.SetAttr("srcs", validSrcs)
				}

				addRule(rule, responses)
			}
		}
	}

	for _, file := range args.RegularFiles {
		if !filesInExistingRules[file] && strings.HasSuffix(file, ".rs") {
			filename := filepath.Base(file)

			// Skip non-crate-root files that belong to a crate
			if filename != "lib.rs" && filesInCrates[file] {
				continue
			}

			response := l.parseFile(args.Config, file, &args)
			if response == nil {
				continue
			}

			inferredKind := l.inferRuleKind(file, dirname, response)

			ruleName := freshRuleName(strings.TrimSuffix(file, ".rs"), existingRuleNames)
			if ruleName == nil {
				l.Log(args.Config, logWarn, args.File, "could not find a suitable rule name, all candidates already taken")
				continue
			}

			srcs := []string{file}
			responses := []*pb.RustImportsResponse{response}

			// For lib.rs crate roots, discover modules and collect all srcs
			if filename == "lib.rs" {
				importsResponses := map[string]*pb.RustImportsResponse{}
				l.discoverModule(args.Config, file, &args, &importsResponses, true)
				srcs = []string{}
				responses = []*pb.RustImportsResponse{}
				for source, sourceResponse := range importsResponses {
					srcs = append(srcs, source)
					if sourceResponse != nil {
						responses = append(responses, sourceResponse)
					}
				}
			}

			newRule := rule.NewRule(inferredKind, *ruleName)
			newRule.SetAttr("srcs", srcs)
			if len(srcs) > 1 {
				newRule.SetAttr("crate_root", file)
			}

			addRule(newRule, responses)
		}
	}

	for _, ruleData := range nonTestRules {
		hasTest := false
		for _, response := range ruleData.responses {
			if response.Hints.HasTest {
				hasTest = true
			}
		}

		existingTestRule := testRules[ruleData.rule.Name()]

		if hasTest {
			// create a corresponding test crate target
			var testRule *rule.Rule
			if existingTestRule == nil {
				testRuleName := freshRuleName(ruleData.rule.Name()+"_test", existingRuleNames)
				if testRuleName == nil {
					l.Log(args.Config, logWarn, args.File, "could not find a suitable test rule name, all candidates already taken")
					continue
				}

				testRule = rule.NewRule("rust_test", *testRuleName)
				testRule.SetAttr("crate", ":"+ruleData.rule.Name())
			} else {
				testRule = CloneRule(existingTestRule)
			}

			result.Gen = append(result.Gen, testRule)
			result.Imports = append(result.Imports, RuleData{
				rule:        testRule,
				responses:   ruleData.responses,
				testedCrate: ruleData.rule,
			})
		} else {
			// TODO: remove test target if we no longer have any tests
		}
	}

	return result
}

func (l *rustLang) parseFile(c *config.Config, file string, args *language.GenerateArgs) *pb.RustImportsResponse {
	request := &pb.RustImportsRequest{FilePath: path.Join(args.Dir, file)}
	response, err := l.Parser.Parse(request)
	if err != nil {
		l.Log(c, logFatal, file, "failed to parse %s: %v", file, err)
	}
	if !response.Success {
		// TODO: It's debatable whether this should be a warning or a fatal error. Having a warning
		// is probably the least surprising, although it could be frustrating to have a bunch of new
		// gazelle errors if there's a parse error in a library that many things depend on.
		l.Log(c, logWarn, file, "failed to parse %s: %s", file, response.ErrorMsg)
		return nil
	}
	return response
}

func (l *rustLang) generateRulesFromCargo(args language.GenerateArgs) language.GenerateResult {
	result := language.GenerateResult{}

	hasBuildScript := false
	parentCrateName := ""
	for _, src := range args.RegularFiles {
		if src == "build.rs" {
			hasBuildScript = true
		}
	}

	for _, file := range args.RegularFiles {
		if file == "Cargo.toml" {
			if response := l.parseCargoToml(args.Config, file, &args); response != nil {
				parentCrateName = response.Name
				if response.Library != nil {
					// if there is a main.rs next to lib.rs, they will both have the same crate
					// name; need to give the library a different name
					suffix := ""
					for _, binary := range response.Binaries {
						if binary.Name == response.Library.Name {
							suffix = "_lib"
							break
						}
					}

					kind := "rust_library"
					if response.Library.ProcMacro {
						kind = "rust_proc_macro"
					}

					l.generateCargoRule(args.Config, &args, response.Library, kind, suffix, []string{}, hasBuildScript, parentCrateName, &result)
				}
				for _, binary := range response.Binaries {
					l.generateCargoRule(args.Config, &args, binary, "rust_binary", "", []string{}, hasBuildScript, parentCrateName, &result)
				}
				for _, test := range response.Tests {
					l.generateCargoRule(args.Config, &args, test, "rust_test", "", []string{}, hasBuildScript, parentCrateName, &result)
				}
				for _, bench := range response.Benches {
					l.generateCargoRule(args.Config, &args, bench, "rust_binary", "", []string{"bench"}, hasBuildScript, parentCrateName, &result)
				}
				for _, example := range response.Examples {
					l.generateCargoRule(args.Config, &args, example, "rust_binary", "", []string{"example"}, hasBuildScript, parentCrateName, &result)
				}
			}
		}
	}

	if hasBuildScript {
		l.generateBuildScript(args.Config, &args, parentCrateName, &result)
	}

	existingRuleNames := make(map[string]bool)
	for _, imp := range result.Imports {
		ruleData := imp.(RuleData)
		existingRuleNames[ruleData.rule.Name()] = true
	}

	for _, imp := range result.Imports {
		ruleData := imp.(RuleData)
		if ruleData.rule.Kind() != "rust_test" {
			hasTest := false
			for _, response := range ruleData.responses {
				if response.Hints.HasTest {
					hasTest = true
				}
			}

			if hasTest {
				testRuleName := freshRuleName(ruleData.rule.Name()+"_test", existingRuleNames)
				if testRuleName == nil {
					l.Log(args.Config, logWarn, args.File, "could not find a suitable test rule name, all candidates already taken")
					continue
				}

				testRule := rule.NewRule("rust_test", *testRuleName)
				testRule.SetAttr("crate", ":"+ruleData.rule.Name())
				testRule.SetAttr("compile_data", []string{"Cargo.toml"})

				result.Gen = append(result.Gen, testRule)
				result.Imports = append(result.Imports, RuleData{
					rule:            testRule,
					responses:       ruleData.responses,
					testedCrate:     ruleData.rule,
					parentCrateName: parentCrateName,
				})
			}
		}
	}

	return result
}

func (l *rustLang) generateCargoRule(c *config.Config, args *language.GenerateArgs,
	crateInfo *pb.CargoCrateInfo, kind string, suffix string, tags []string,
	hasBuildScript bool, parentCrateName string, result *language.GenerateResult) {

	targetName := crateInfo.Name + suffix
	crateName := crateInfo.Name

	var crateRoot *string = nil
	if len(crateInfo.Srcs) == 1 {
		onlySrc := crateInfo.Srcs[0]
		onlySrcFilename := filepath.Base(onlySrc)
		// handle cases where we need to specify the crate root manually
		if !(kind == "rust_library" && onlySrcFilename == "lib.rs") &&
			!((kind == "rust_binary" || kind == "rust_test") && onlySrcFilename == "main.rs") {
			crateRoot = &onlySrc
		}
	}

	// traverse all files we know about to determine the full module structure
	importsResponses := map[string]*pb.RustImportsResponse{}
	for _, src := range crateInfo.Srcs {
		// It is possible for declared files to be absent if they are
		// supposed to be produced by the build script of the crate.
		if fileExists(src, args) {
			l.discoverModule(c, src, args, &importsResponses, true)
		}
	}

	srcs := []string{}
	responses := []*pb.RustImportsResponse{}

	for src, response := range importsResponses {
		srcs = append(srcs, src)
		if response != nil {
			responses = append(responses, response)
		}
	}

	newRule := rule.NewRule(kind, targetName)

	if len(srcs) > 0 {
		newRule.SetAttr("srcs", srcs)
	}
	newRule.SetAttr("visibility", []string{"//visibility:public"})
	newRule.SetAttr("compile_data", []string{"Cargo.toml"})

	if targetName != crateName {
		newRule.SetAttr("crate_name", crateName)
	}

	if len(tags) != 0 {
		newRule.SetAttr("tags", tags)
	}

	if crateRoot != nil && len(srcs) > 1 {
		newRule.SetAttr("crate_root", *crateRoot)
	}

	var buildScript *label.Label = nil
	if hasBuildScript && (kind == "rust_library" || kind == "rust_binary") {
		build_script_label, err := label.Parse(":build_script")
		if err != nil {
			l.Log(c, logFatal, "build.rs", "bad build script label: %v\n", err)
		}
		buildScript = &build_script_label
	}

	result.Gen = append(result.Gen, newRule)
	result.Imports = append(result.Imports, RuleData{
		rule:            newRule,
		responses:       responses,
		testedCrate:     nil,
		buildScript:     buildScript,
		parentCrateName: parentCrateName,
	})
}

func (l *rustLang) generateBuildScript(c *config.Config, args *language.GenerateArgs,
	crateName string, result *language.GenerateResult) {
	importsResponses := map[string]*pb.RustImportsResponse{}
	l.discoverModule(c, "build.rs", args, &importsResponses, true)

	srcs := []string{}
	responses := []*pb.RustImportsResponse{}

	for src, response := range importsResponses {
		srcs = append(srcs, src)
		if response != nil {
			responses = append(responses, response)
		}
	}

	newRule := rule.NewRule("cargo_build_script", "build_script")
	newRule.SetAttr("srcs", srcs)
	newRule.SetAttr("visibility", []string{"//visibility:public"})
	newRule.SetAttr("compile_data", []string{"Cargo.toml"})
	newRule.SetAttr("crate_root", "build.rs")

	result.Gen = append(result.Gen, newRule)
	result.Imports = append(result.Imports, RuleData{
		rule:            newRule,
		responses:       responses,
		testedCrate:     nil,
		parentCrateName: crateName,
	})
}

func (l *rustLang) discoverModule(c *config.Config, file string, args *language.GenerateArgs,
	importsResponses *map[string]*pb.RustImportsResponse, isModRoot bool) {

	if _, ok := (*importsResponses)[file]; ok {
		return
	}

	response := l.parseFile(c, file, args)
	(*importsResponses)[file] = response

	if response != nil {
		dirname := filepath.Dir(file)
		currentModName := strings.TrimSuffix(filepath.Base(file), ".rs")

		for _, externMod := range response.ExternMods {
			var externModPath string
			var childIsModRoot bool

			if isModRoot {
				// first check for an adjacent file
				externModPath = filepath.Join(dirname, externMod+".rs")
				childIsModRoot = false

				// then check for an equivalent mod.rs
				if !fileExists(externModPath, args) {
					externModPath = filepath.Join(dirname, externMod, "mod.rs")
					childIsModRoot = true
				}
			} else {
				// look in the subdirectory for the current module
				externModPath = filepath.Join(dirname, currentModName, externMod+".rs")
				childIsModRoot = false
			}

			if !fileExists(externModPath, args) {
				l.Log(c, logWarn, file, "could not find file for mod %s", externMod)
				continue
			}

			l.discoverModule(c, externModPath, args, importsResponses, childIsModRoot)
		}
	}
}

func (l *rustLang) parseCargoToml(c *config.Config, file string, args *language.GenerateArgs) *pb.CargoTomlResponse {
	request := &pb.CargoTomlRequest{FilePath: path.Join(args.Dir, file)}
	response, err := l.Parser.ParseCargoToml(request)
	if err != nil {
		l.Log(c, logFatal, file, "failed to parse Cargo.toml: %v", err)
	}
	if !response.Success {
		l.Log(c, logWarn, file, "failed to parse Cargo.toml: %s", response.ErrorMsg)
		return nil
	}
	return response
}

func fileExists(path string, args *language.GenerateArgs) bool {
	fullPath := filepath.Join(args.Dir, path)
	_, err := os.Stat(fullPath)
	return err == nil
}

// Returns the set of files in this directory that are claimed by a parent lib.rs crate.
// This includes mod.rs (if this directory is a module) and any files declared via `mod`
// in mod.rs.
func (l *rustLang) getFilesClaimedByParent(c *config.Config, args *language.GenerateArgs) map[string]bool {
	claimed := make(map[string]bool)

	rel := args.Rel
	if rel == "" || rel == "." {
		return claimed
	}

	pathComponents := strings.Split(filepath.ToSlash(rel), "/")

	// Walk up to find the nearest ancestor with lib.rs
	for ancestorDepth := len(pathComponents) - 1; ancestorDepth >= 0; ancestorDepth-- {
		var ancestorRel string
		if ancestorDepth == 0 {
			ancestorRel = ""
		} else {
			ancestorRel = strings.Join(pathComponents[:ancestorDepth], "/")
		}
		ancestorDir := filepath.Join(c.RepoRoot, ancestorRel)

		libPath := filepath.Join(ancestorDir, "lib.rs")
		if _, err := os.Stat(libPath); err == nil {
			// Found a lib.rs ancestor. Verify the module chain and get claimed files.
			l.getClaimedFilesFromChain(c, ancestorDir, ancestorRel, pathComponents[ancestorDepth:], args, claimed)
			return claimed
		}
	}

	return claimed
}

// Verifies there's a valid chain of mod declarations from startDir through each
// component in pathComponents, and populates the claimed map with files in the target
// directory that are part of the module tree.
func (l *rustLang) getClaimedFilesFromChain(c *config.Config, startDir, startRel string,
	pathComponents []string, args *language.GenerateArgs, claimed map[string]bool) {

	if len(pathComponents) == 0 {
		return
	}

	currentDir := startDir
	currentRel := startRel
	currentFile := "lib.rs"

	for i, component := range pathComponents {
		parseArgs := &language.GenerateArgs{
			Config: args.Config,
			Dir:    currentDir,
			Rel:    currentRel,
		}

		response := l.parseFile(c, currentFile, parseArgs)
		if response == nil {
			return
		}

		// Check if this file declares `mod <component>;`
		found := false
		for _, externMod := range response.ExternMods {
			if externMod == component {
				found = true
				break
			}
		}

		if !found {
			return
		}

		// If this is the last component, we've reached the target directory
		if i == len(pathComponents)-1 {
			// Check for mod.rs style (old) or adjacent component.rs style (Rust 2018+)
			// We accept both without requiring edition configuration.
			targetDir := filepath.Join(currentDir, component)
			modRsPath := filepath.Join(targetDir, "mod.rs")
			adjacentModPath := filepath.Join(currentDir, component+".rs")

			if _, err := os.Stat(modRsPath); err == nil {
				// Old style: component/mod.rs is the module file
				claimed["mod.rs"] = true
				l.claimDeclaredSubmodules(c, targetDir, "mod.rs", claimed)
			} else if _, err := os.Stat(adjacentModPath); err == nil {
				// Rust 2018+ style: component.rs (in parent dir) is the module file,
				// but its submodules live in component/ directory
				l.claimDeclaredSubmodules(c, targetDir, adjacentModPath, claimed)
			}
			return
		}

		// For intermediate directories, check both module styles
		modRsPath := filepath.Join(currentDir, component, "mod.rs")
		adjacentModPath := filepath.Join(currentDir, component+".rs")

		if _, err := os.Stat(modRsPath); err == nil {
			// Old style: component/mod.rs
			currentDir = filepath.Join(currentDir, component)
			currentRel = path.Join(currentRel, component)
			currentFile = "mod.rs"
		} else if _, err := os.Stat(adjacentModPath); err == nil {
			// Rust 2018+ style: component.rs (stays in same directory for parsing)
			currentFile = component + ".rs"
			// Note: for next iteration, we still descend into the component directory
			// because that's where submodules live
			currentDir = filepath.Join(currentDir, component)
			currentRel = path.Join(currentRel, component)
		} else {
			// Neither exists, chain is broken
			return
		}
	}
}

// claimDeclaredSubmodules parses a module file and claims any .rs files in targetDir
// that it declares via `mod` statements. The moduleFile can be either:
// - A filename relative to targetDir (e.g., "mod.rs" for old style)
// - An absolute path (e.g., "/path/to/subdir.rs" for Rust 2018+ style)
func (l *rustLang) claimDeclaredSubmodules(c *config.Config, targetDir, moduleFile string, claimed map[string]bool) {
	var parseDir, parseFile string

	if filepath.IsAbs(moduleFile) {
		parseDir = filepath.Dir(moduleFile)
		parseFile = filepath.Base(moduleFile)
	} else {
		parseDir = targetDir
		parseFile = moduleFile
	}

	parseArgs := &language.GenerateArgs{
		Config: c,
		Dir:    parseDir,
		Rel:    "", // Not used for parsing
	}
	response := l.parseFile(c, parseFile, parseArgs)
	if response != nil {
		for _, externMod := range response.ExternMods {
			// Check if this submodule is an adjacent file in targetDir
			adjacentFile := externMod + ".rs"
			adjacentPath := filepath.Join(targetDir, adjacentFile)
			if _, err := os.Stat(adjacentPath); err == nil {
				claimed[adjacentFile] = true
			}
		}
	}
}
