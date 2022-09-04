package rust_language

import (
	"errors"
	"fmt"
	"io"
	"log"
	"os"
	"os/exec"

	"github.com/bazelbuild/rules_go/go/tools/bazel"

	"google.golang.org/protobuf/encoding/protowire"
	"google.golang.org/protobuf/proto"

	pb "github.com/calsign/gazelle_rust/proto"
)

type Parser struct {
	cmd            *exec.Cmd
	stdin          io.WriteCloser
	stdout         io.ReadCloser
	marshalOptions *proto.MarshalOptions
}

func NewParser() *Parser {
	path, err := bazel.Runfile("rust_parser/rust_parser")
	if err != nil {
		log.Fatal(err)
	}
	cmd := exec.Command(path, "stream-proto")
	stdin, err := cmd.StdinPipe()
	if err != nil {
		log.Fatal(err)
	}
	stdout, err := cmd.StdoutPipe()
	if err != nil {
		log.Fatal(err)
	}
	cmd.Stderr = os.Stderr
	cmd.Start()
	return &Parser{
		cmd:            cmd,
		stdin:          stdin,
		stdout:         stdout,
		marshalOptions: &proto.MarshalOptions{},
	}
}

const maxMsgSize uint32 = 4096

var buf []byte = make([]byte, maxMsgSize)
var sf32 int = protowire.SizeFixed32()

func (p *Parser) WriteRequest(request *pb.RustImportsRequest) error {
	size := p.marshalOptions.Size(request)

	buf := []byte{}
	buf = protowire.AppendFixed32(buf, uint32(size))
	buf, err := p.marshalOptions.MarshalAppend(buf, request)
	if err != nil {
		return err
	}

	p.stdin.Write(buf)

	return nil
}

func (p *Parser) ReadResponse() (*pb.RustImportsResponse, error) {
	n, err := p.stdout.Read(buf[:sf32])
	if n != sf32 {
		return nil, errors.New("invalid size")
	}
	if err != nil {
		return nil, err
	}
	size, n := protowire.ConsumeFixed32(buf[:sf32])
	if n != sf32 {
		log.Fatal("n: %v\n", n)
	}
	if size > maxMsgSize {
		return nil, errors.New(fmt.Sprintf(
			"message size %d exceeds max message size %d", size, maxMsgSize))
	}
	n, err = p.stdout.Read(buf[:size])
	if err != nil {
		return nil, err
	}
	if uint32(n) != size {
		return nil, errors.New(fmt.Sprintf(
			"recieved wrong size %d, expected %d", n, size))
	}
	response := &pb.RustImportsResponse{}
	err = proto.Unmarshal(buf[:size], response)
	if err != nil {
		return nil, err
	}

	return response, nil
}

func (p *Parser) Parse(request *pb.RustImportsRequest) (*pb.RustImportsResponse, error) {
	if err := p.WriteRequest(request); err != nil {
		return nil, err
	}
	response, err := p.ReadResponse()
	if err != nil {
		return nil, err
	}
	return response, nil
}
