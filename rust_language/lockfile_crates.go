package rust_language

import (
	"log"

	"github.com/bazelbuild/bazel-gazelle/resolve"

	pb "github.com/calsign/gazelle_rust/proto"
)

type LockfileCrates struct {
	Crates map[resolve.ImportSpec]bool
}

func NewLockfileCrates(p *Parser, lockfilePath string) *LockfileCrates {
	lockfileCrates := &LockfileCrates{
		Crates: make(map[resolve.ImportSpec]bool),
	}

	request := &pb.LockfileCratesRequest{LockfilePath: lockfilePath}
	response, err := p.GetLockfileCrates(request)
	if err != nil {
		log.Fatal(err)
	}

	for _, crate := range response.Crates {
		spec := resolve.ImportSpec{
			Lang: langName,
			Imp:  crate,
		}
		lockfileCrates.Crates[spec] = true
	}
	// TODO: proc macro crates

	return lockfileCrates
}
