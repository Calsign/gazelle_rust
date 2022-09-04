package rust_language

import (
	"log"

	"github.com/bazelbuild/bazel-gazelle/resolve"

	pb "github.com/calsign/gazelle_rust/proto"
)

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
		// TODO: support proc_macros
		spec := resolve.ImportSpec{
			Lang: langName,
			Imp:  crate.CrateName,
		}
		lockfileCrates.Crates[spec] = crate.Name
	}

	return lockfileCrates
}
