extends Control

func _ready():
	var gdn = GDNative.new()
	gdn.library = load("res://librust.gdnlib")
	gdn.initialize()
	var result = gdn.call_native("standard_varcall", "add_42", [13])
	print("13 + 42 = " + str(result))
	gdn.terminate()
	get_tree().quit()
