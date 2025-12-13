package rust_language

import (
	"github.com/bazelbuild/bazel-gazelle/config"
	"github.com/bazelbuild/bazel-gazelle/resolve"

	pb "github.com/calsign/gazelle_rust/proto"
)

// For cargo lockfiles, we guess whether each crate is a proc_macro by checking its dependencies for
// proc-macro or proc-macro2. Sometimes this is wrong. For this purpose, we have this mapping of
// known overrides. The user can also specify additional overrides with a directive.
var procMacroOverrides map[string]bool = map[string]bool{
	"syn": false,
}

type LockfileCrates struct {
	// map from imports to package names
	Crates map[resolve.ImportSpec]string
	// map from package names to whether they are multiversion dependencies
	Multiversion map[string]bool
	// dependencies per crate, the inner map is names to versions
	DependenciesPerCrate map[string](map[string]string)
	// track which crates have been used so that we can report unused crates
	UsedCrates map[string]bool
}

func EmptyLockfileCrates() *LockfileCrates {
	return &LockfileCrates{
		Crates:               make(map[resolve.ImportSpec]string),
		Multiversion:         make(map[string]bool),
		DependenciesPerCrate: make(map[string](map[string]string)),
		UsedCrates:           make(map[string]bool),
	}
}

func (l *rustLang) NewLockfileCrates(c *config.Config, lockfilePath string, cargo bool) *LockfileCrates {
	lockfileCrates := EmptyLockfileCrates()

	var request *pb.LockfileCratesRequest
	if cargo {
		request = &pb.LockfileCratesRequest{
			Lockfile: &pb.LockfileCratesRequest_CargoLockfilePath{
				CargoLockfilePath: lockfilePath,
			},
		}
	} else {
		request = &pb.LockfileCratesRequest{
			Lockfile: &pb.LockfileCratesRequest_LockfilePath{
				LockfilePath: lockfilePath,
			},
		}
	}

	response, err := l.Parser.GetLockfileCrates(request)
	if err != nil {
		l.Log(c, logFatal, lockfilePath, "failed to parse lockfile crates: %v", err)
	}

	var requestedVersions = make(map[string](map[string]bool))

	for _, crate := range response.Crates {
		is_proc_macro := crate.ProcMacro
		if proc_macro, ok := procMacroOverrides[crate.CrateName]; ok {
			// well-known override
			// NOTE: user-defined overrides are handled in resolve.go
			is_proc_macro = proc_macro
		}

		var lang string
		if is_proc_macro {
			lang = procMacroLangName
		} else {
			lang = langName
		}

		spec := resolve.ImportSpec{
			Lang: lang,
			Imp:  crate.CrateName,
		}
		lockfileCrates.Crates[spec] = crate.Name

		if crate.WorkspaceMember {
			var dependencies = make(map[string]string)
			for _, dep := range crate.Dependencies {
				dependencies[dep.Name] = dep.Version
				if _, ok := requestedVersions[dep.Name]; !ok {
					requestedVersions[dep.Name] = make(map[string]bool)
				}
				requestedVersions[dep.Name][dep.Version] = true
			}
			lockfileCrates.DependenciesPerCrate[crate.Name] = dependencies
		}
	}

	for crate, versions := range requestedVersions {
		lockfileCrates.Multiversion[crate] = len(versions) > 1
	}

	return lockfileCrates
}

func (l *LockfileCrates) UnusedCrates(allowedUnusedCrates map[string]bool) []string {
	unusedCrates := []string{}

	for _, crate := range l.Crates {
		if !l.UsedCrates[crate] && !allowedUnusedCrates[crate] {
			unusedCrates = append(unusedCrates, crate)
		}
	}

	return unusedCrates
}
