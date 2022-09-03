package rust_language

import (
	"fmt"
	"log"
	"path"
	"strings"

	"github.com/bazelbuild/bazel-gazelle/language"

	pb "github.com/calsign/gazelle_rust/proto"
)

func (l *rustLang) GenerateRules(args language.GenerateArgs) language.GenerateResult {
	result := language.GenerateResult{}

	for _, file := range args.RegularFiles {
		if strings.HasSuffix(file, ".rs") {
			path := path.Join(args.Dir, file)
			request := &pb.RustImportsRequest{FilePath: path}

			if err := l.Parser.WriteRequest(request); err != nil {
				log.Fatal(err)
			}
			response, err := l.Parser.ReadResponse()
			if err != nil {
				log.Fatal(err)
			}
			imports := response.GetImports()
			fmt.Printf("imports: %v\n", imports)
		}
	}

	return result
}
