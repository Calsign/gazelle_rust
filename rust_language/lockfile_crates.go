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

type LockfileCrate struct {
	Name string
	Versions []string
}

type LockfileCrates struct {
	Crates map[resolve.ImportSpec]LockfileCrate
	// track which crates have been used so that we can report unused crates
	UsedCrates map[string]bool
}

func EmptyLockfileCrates() *LockfileCrates {
	return &LockfileCrates{
		Crates:     make(map[resolve.ImportSpec]LockfileCrate),
		UsedCrates: make(map[string]bool),
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
		existingLockfileCrate, ok := lockfileCrates.Crates[spec]
		if ok {
			existingLockfileCrate.Versions = append(existingLockfileCrate.Versions, crate.Version)
			lockfileCrates.Crates[spec] = existingLockfileCrate
		} else {
			lockfileCrate := LockfileCrate{
				Name: crate.Name,
				Versions: []string{crate.Version},
			}
			lockfileCrates.Crates[spec] = lockfileCrate
		}

	}

	return lockfileCrates
}

func (l *LockfileCrates) UnusedCrates(allowedUnusedCrates map[string]bool) []string {
	unusedCrates := []string{}

	for _, crate := range l.Crates {
		if !l.UsedCrates[crate.Name] && !allowedUnusedCrates[crate.Name] {
			unusedCrates = append(unusedCrates, crate.Name)
		}
	}

	return unusedCrates
}
