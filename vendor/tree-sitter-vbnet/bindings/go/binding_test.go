package tree_sitter_vb_dotnet_test

import (
	"testing"

	tree_sitter "github.com/smacker/go-tree-sitter"
	"github.com/tree-sitter/tree-sitter-vb_dotnet"
)

func TestCanLoadGrammar(t *testing.T) {
	language := tree_sitter.NewLanguage(tree_sitter_vb_dotnet.Language())
	if language == nil {
		t.Errorf("Error loading VbDotnet grammar")
	}
}
