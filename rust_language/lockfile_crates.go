package rust_language

import (
	"log"

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
	Crates map[resolve.ImportSpec]string
}

func EmptyLockfileCrates() *LockfileCrates {
	return &LockfileCrates{
		Crates: make(map[resolve.ImportSpec]string),
	}
}

func NewLockfileCrates(p *Parser, lockfilePath string, cargo bool) *LockfileCrates {
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

	response, err := p.GetLockfileCrates(request)
	if err != nil {
		log.Fatal(err)
	}

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
	}

	return lockfileCrates
}
