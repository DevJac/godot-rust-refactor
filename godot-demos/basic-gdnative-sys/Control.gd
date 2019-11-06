extends Control

func _ready():
	var gdn = GDNative.new()
	gdn.library = load("res://librust.gdnlib")
	gdn.initialize()
	var result = gdn.call_native("standard_varcall", "add_42", [13])
	print("13 + 42 = " + str(result))
	gdn.terminate()

	var simple = load("res://simple.gdns")
	var a = simple.new()
	var b = simple.new()
	var c = simple.new()
	print(a.get_string())
	print(b.get_string())
	print(c.get_string())
	a.set_string("A");
	print(a.get_string())
	print(b.get_string())
	print(c.get_string())
	a.set_string("a");
	b.set_string("b");
	print(a.get_string())
	print(b.get_string())
	print(c.get_string())
	a.set_string("A");
	b.set_string("B");
	c.set_string("C");
	print(a.get_string())
	print(b.get_string())
	print(c.get_string())

	get_tree().quit()
