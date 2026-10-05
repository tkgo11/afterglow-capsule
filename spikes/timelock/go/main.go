// SPIKE ONLY: a strict adapter around the official Go tlock reference.
package main

import (
	"bytes"
	"encoding/hex"
	"encoding/json"
	"errors"
	"fmt"
	"os"
	"strconv"

	"github.com/drand/tlock"
	"github.com/drand/tlock/networks/fixed"
)

type fixture struct {
	FormatName           string `json:"format_name"`
	FormatVersion        uint16 `json:"format_version"`
	MinimumReaderVersion uint16 `json:"minimum_reader_version"`
	Chain                struct {
		PublicKey   string `json:"public_key"`
		Period      uint64 `json:"period"`
		GenesisTime int64  `json:"genesis_time"`
		Hash        string `json:"hash"`
		SchemeID    string `json:"schemeID"`
	} `json:"chain"`
}

func run() error {
	if len(os.Args) < 3 {
		return errors.New("usage: spike-a-reference encrypt|decrypt fixture ...")
	}
	fixtureBytes, err := os.ReadFile(os.Args[2])
	if err != nil {
		return err
	}
	var f fixture
	if err := json.Unmarshal(fixtureBytes, &f); err != nil {
		return err
	}
	if f.FormatName != "afterglow-spike-a-public-fixture" || f.FormatVersion != 1 || f.MinimumReaderVersion != 1 {
		return errors.New("unsupported fixture format")
	}
	infoBytes, err := json.Marshal(map[string]any{
		"public_key": f.Chain.PublicKey, "period": f.Chain.Period, "genesis_time": f.Chain.GenesisTime,
		"chain_hash": f.Chain.Hash, "scheme": f.Chain.SchemeID,
	})
	if err != nil {
		return err
	}
	network, err := fixed.FromInfo(string(infoBytes))
	if err != nil {
		return err
	}
	engine := tlock.New(network).Strict()
	switch {
	case os.Args[1] == "encrypt" && len(os.Args) == 6:
		plaintext, err := os.ReadFile(os.Args[3])
		if err != nil {
			return err
		}
		round, err := strconv.ParseUint(os.Args[5], 10, 64)
		if err != nil || round == 0 {
			return errors.New("invalid round")
		}
		var ciphertext bytes.Buffer
		if err := engine.Encrypt(&ciphertext, bytes.NewReader(plaintext), round); err != nil {
			return err
		}
		return os.WriteFile(os.Args[4], ciphertext.Bytes(), 0600)
	case os.Args[1] == "decrypt" && len(os.Args) == 6:
		ciphertext, err := os.ReadFile(os.Args[3])
		if err != nil {
			return err
		}
		beaconBytes, err := os.ReadFile(os.Args[5])
		if err != nil {
			return err
		}
		var beacon struct {
			Signature string `json:"signature"`
		}
		if err := json.Unmarshal(beaconBytes, &beacon); err != nil {
			return err
		}
		signature, err := hex.DecodeString(beacon.Signature)
		if err != nil {
			return err
		}
		network.SetSignature(signature)
		var plaintext bytes.Buffer
		if err := engine.Decrypt(&plaintext, bytes.NewReader(ciphertext)); err != nil {
			return err
		}
		return os.WriteFile(os.Args[4], plaintext.Bytes(), 0600)
	default:
		return errors.New("invalid command or argument count")
	}
}

func main() {
	if err := run(); err != nil {
		fmt.Fprintln(os.Stderr, "Spike A reference:", err)
		os.Exit(1)
	}
}
