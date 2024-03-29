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

var buf []byte = make([]byte, 1024)
var sf32 int = protowire.SizeFixed32()

func (p *Parser) WriteRequest(request *pb.Request) error {
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

func ReadResponse[M proto.Message](p *Parser, response M) error {
	n, err := io.ReadFull(p.stdout, buf[:sf32])
	if n != sf32 {
		return errors.New("invalid size")
	}
	if err != nil {
		return err
	}
	size, n := protowire.ConsumeFixed32(buf[:sf32])
	if n != sf32 {
		log.Fatalf("n: %v\n", n)
	}
	if size > uint32(len(buf)) {
		// grow buffer as neeeded
		buf = make([]byte, size)
	}
	n, err = io.ReadFull(p.stdout, buf[:size])
	if err != nil {
		return err
	}
	if uint32(n) != size {
		return errors.New(fmt.Sprintf(
			"recieved wrong size %d, expected %d", n, size))
	}
	err = proto.Unmarshal(buf[:size], response)
	if err != nil {
		return err
	}

	return nil
}

func (p *Parser) Parse(request *pb.RustImportsRequest) (*pb.RustImportsResponse, error) {
	if err := p.WriteRequest(&pb.Request{
		Kind: &pb.Request_RustImports{RustImports: request}}); err != nil {
		return nil, err
	}
	response := &pb.RustImportsResponse{}
	if err := ReadResponse[*pb.RustImportsResponse](p, response); err != nil {
		return nil, err
	}
	return response, nil
}

func (p *Parser) GetLockfileCrates(request *pb.LockfileCratesRequest) (*pb.LockfileCratesResponse, error) {
	if err := p.WriteRequest(&pb.Request{
		Kind: &pb.Request_LockfileCrates{LockfileCrates: request}}); err != nil {
		return nil, err
	}
	response := &pb.LockfileCratesResponse{}
	if err := ReadResponse[*pb.LockfileCratesResponse](p, response); err != nil {
		return nil, err
	}
	return response, nil
}

func (p *Parser) ParseCargoToml(request *pb.CargoTomlRequest) (*pb.CargoTomlResponse, error) {
	if err := p.WriteRequest(&pb.Request{
		Kind: &pb.Request_CargoToml{CargoToml: request}}); err != nil {
		return nil, err
	}
	response := &pb.CargoTomlResponse{}
	if err := ReadResponse[*pb.CargoTomlResponse](p, response); err != nil {
		return nil, err
	}
	return response, nil
}
