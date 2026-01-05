package main

import "core:fmt"

// Test program for the Codebook spell-checking tool.
// The goal is to spellcheck comments, strings,
// identifiers during declarations, but not during usage

// Commennt
/*
	Block cooment
	/*
		Netsed block
	*/
*/

my_proecdure :: proc(my_prameter: int, another_paramter: f64) {
	fmt.println(cast(f64)my_prameter + another_paramter)
}

another_porocedure :: proc(my_prameter: f64, another_paramter: int) {
	fmt.println(my_prameter + cast(f64)another_paramter)
}

overloded_procedure:: proc{my_proecdure, another_porocedure}

with_deafult :: proc(my_prameter:= 42) {
	fmt.println(my_prameter)
}

with_varidic :: proc(numberes: ..int) {
	fmt.println(numberes)
}

MY_CONSATANT : int : 123
ANOTHER_COONSTANT :: 456

main :: proc() {
	declaring_without_assignement: int
	declaring_anotther, and_annother: int
	with_assignement := 42
	assignement_with_explicit_type : int = 33
	and_another_one, and_more := "Helloep", "Wordl"
	fmt.println(
		MY_CONSATANT,
		ANOTHER_COONSTANT,
		declaring_without_assignement,
		declaring_anotther,
		with_assignement,
		assignement_with_explicit_type,
		and_another_one,
		and_more,
	)

	MyAwseomeStruct :: struct {
		my_field: f32,
		another_field: f32,
	}
	foo := MyAwseomeStruct{1, 2}
	fmt.println(foo.my_field, foo.another_field)

	CompacotStruct :: struct {
		aples, banananas, ornages: int
	}
	bar := CompacotStruct{3, 4, 5}
	fmt.println(bar.aples, bar.banananas, bar.ornages)

	TWOOF :: 2
	MyCratfyEnum :: enum {
		Aapple,
		Baanana = 2,
		Oranege = TWOOF,
	}
	buzz := MyCratfyEnum.Baanana
	fmt.println(buzz)

	MyUnberakableUnion :: union {int, bool}

	MyFruttyInstruction :: bit_field u64 {
		verison: u8         | 3,
		ttl: u8             | 8,
		fruit: MyCratfyEnum | TWOOF,
	 	opration: u8        | 3,
		left_opernd: u16    | 16,
		right_oprand: u16   | 16,
		destination: u16    | 16,
	}
	i := MyFruttyInstruction{}
	i.fruit = .Baanana
	fmt.println(i.left_opernd, i.right_oprand)

	fmt.println("Helolo, Wlorld!")

	overloded_procedure(33, 3.3)
	overloded_procedure(4, 44.4)
	with_deafult(42)
	with_varidic(1, 2, 3)
}
