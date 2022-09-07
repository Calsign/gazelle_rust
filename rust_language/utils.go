package rust_language

import (
	"fmt"
	"log"

	"github.com/bazelbuild/bazel-gazelle/config"
	"github.com/bazelbuild/bazel-gazelle/label"
	"github.com/bazelbuild/bazel-gazelle/rule"
)

type logLevel int

const (
	logFatal logLevel = iota
	logErr
	logWarn
	logInfo
)

func (l *rustLang) Log(c *config.Config, level logLevel, from interface{}, msg string, args ...interface{}) {
	fmtMsg := fmt.Sprintf(msg, args...)

	var fromStr string
	switch f := from.(type) {
	case label.Label:
		fromStr = f.String()
	case string:
		fromStr = f
	case *rule.File:
		if f != nil {
			fromStr = f.Path
		} else {
			fromStr = ""
		}
	default:
		log.Panicf("unsupported from type: %v", from)
	}

	if fromStr != "" {
		fromStr = fmt.Sprintf("%s: ", fromStr)
	}

	if level == logFatal || (level != logInfo && c.Strict) {
		log.Fatalf("%s%s", fromStr, fmtMsg)
	} else {
		log.Printf("%s%s", fromStr, fmtMsg)
	}
}

func SliceContains(slice []string, value string) bool {
	for _, item := range slice {
		if item == value {
			return true
		}
	}
	return false
}
