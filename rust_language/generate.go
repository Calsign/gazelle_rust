package rust_language

import (
	"log"
	"path"
	"strings"

	"github.com/bazelbuild/bazel-gazelle/language"
	"github.com/bazelbuild/bazel-gazelle/rule"

	pb "github.com/calsign/gazelle_rust/proto"
)

func (l *rustLang) GenerateRules(args language.GenerateArgs) language.GenerateResult {
	result := language.GenerateResult{}

	filesInExistingRules := map[string]bool{}

	for _, rule := range args.File.Rules {
		responses := []*pb.RustImportsResponse{}

		for _, file := range rule.AttrStrings("srcs") {
			filesInExistingRules[file] = true

			if strings.HasSuffix(file, ".rs") {
				response := l.parseFile(file, args)
				responses = append(responses, response)
			}
		}

		result.Gen = append(result.Gen, rule)
		result.Imports = append(result.Imports, responses)
	}

	for _, file := range args.RegularFiles {
		if !filesInExistingRules[file] && strings.HasSuffix(file, ".rs") {
			response := l.parseFile(file, args)

			rule := rule.NewRule("rust_library", strings.TrimSuffix(file, ".rs"))
			rule.SetAttr("srcs", []string{file})

			result.Gen = append(result.Gen, rule)
			result.Imports = append(result.Imports, []*pb.RustImportsResponse{response})
		}
	}

	return result
}

func (l *rustLang) parseFile(file string, args language.GenerateArgs) *pb.RustImportsResponse {
	request := &pb.RustImportsRequest{FilePath: path.Join(args.Dir, file)}
	response, err := l.Parser.Parse(request)
	if err != nil {
		log.Fatal(err)
	}
	return response
}
