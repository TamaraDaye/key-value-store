// Package database
package main


type ItemPointer uint32


// PageHeader will store metadata about each page in the file
type PageHeader struct {
	pdChecksum uint16
	pdFlags uint16
	pdLower uint16
	pdUpper uint16
	pdPageSize uint16
}

type KVHeader struct {
	key  uint16
	value uint16
}


func (ip ItemPointer) Offset()  uint16 {
	return uint16(ip & 0x7fff)
}

func(ip ItemPointer) Flags() uint8 {
	return uint8((ip >> 15 ) & 0x3)
}

func(ip ItemPointer) Length() uint16 {
	return uint16((ip >> 17) & 0x7fff)
}


