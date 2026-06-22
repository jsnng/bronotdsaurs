package main

import (
	"context"
	"encoding/binary"
	"fmt"
	"io"
	"os"

	"github.com/AzureAD/microsoft-authentication-library-for-go/apps/confidential"
	"github.com/joho/godotenv"
)

func init() {
	_ = godotenv.Load() // ignore if missing .env
}

func readFrame(r io.Reader) ([]byte, error) {
	var lenbuf [4]byte
	if _, err := io.ReadFull(r, lenbuf[:]); err != nil {
		return nil, err
	}
	n := binary.LittleEndian.Uint32(lenbuf[:])
	buf := make([]byte, n)
	if _, err := io.ReadFull(r, buf); err != nil {
		return nil, err
	}
	return buf, nil
}

func run() error {
	in := os.Stdin

	stsBytes, err := readFrame(in)
	if err != nil {
		return fmt.Errorf("reading sts_url: %w", err)
	}
	spnBytes, err := readFrame(in)
	if err != nil {
		return fmt.Errorf("reading spn: %w", err)
	}

	var hasNonce [1]byte
	if _, err := io.ReadFull(in, hasNonce[:]); err != nil {
		return fmt.Errorf("reading nonce flag: %w", err)
	}
	if hasNonce[0] == 1 {
		var nonce [32]byte
		if _, err := io.ReadFull(in, nonce[:]); err != nil {
			return fmt.Errorf("reading nonce: %w", err)
		}
		_ = nonce
	}

	sts := string(stsBytes)
	scope := string(spnBytes)

	credential, err := confidential.NewCredFromSecret(os.Getenv("AZUREAD_CLIENT_SECRET"))
	if err != nil {
		return fmt.Errorf("creating credential: %w", err)
	}
	client, err := confidential.New(sts, os.Getenv("AZUREAD_APPLICATION_ID"), credential)
	if err != nil {
		return fmt.Errorf("creating client: %w", err)
	}
	result, err := client.AcquireTokenByCredential(context.Background(), []string{scope})
	if err != nil {
		return fmt.Errorf("acquiring token: %w", err)
	}

	if _, err := os.Stdout.Write([]byte(result.AccessToken)); err != nil {
		return fmt.Errorf("writing token: %w", err)
	}
	return nil
}

func main() {
	if err := run(); err != nil {
		fmt.Fprintln(os.Stderr, "fedauth:", err)
		os.Exit(1)
	}
}
