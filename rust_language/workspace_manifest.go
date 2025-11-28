package rust_language

import (
	"github.com/bazelbuild/bazel-gazelle/config"

	pb "github.com/calsign/gazelle_rust/proto"
)

type WorkspaceDependencyInfo struct {
	Version string
	Path string
}

type WorkspaceManifest struct {
	Dependencies map[string]WorkspaceDependencyInfo
}

func EmptyWorkspaceManifest() *WorkspaceManifest {
	return &WorkspaceManifest{
		Dependencies: make(map[string]WorkspaceDependencyInfo),
	}
}

func (l *rustLang) NewWorkspaceManifest(c *config.Config, workspacePath string) * WorkspaceManifest {
	workspaceManifest := EmptyWorkspaceManifest()

	request := &pb.CargoWorkspaceRequest{
		CargoWorkspacePath: workspacePath,
	}
	response, err := l.Parser.GetWorkspaceManifest(request)
	if err != nil {
		l.Log(c, logFatal, workspacePath, "failed to parse cargo workspace manifest: %v", err)
	}
	for _, dep := range response.Deps {
		dependencyInfo := WorkspaceDependencyInfo{
			Version: dep.Version,
			Path: dep.Path,
		}
		workspaceManifest.Dependencies[dep.Name] = dependencyInfo
	}

	return workspaceManifest
}
