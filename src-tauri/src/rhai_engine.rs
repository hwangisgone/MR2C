use rhai::{Engine, Scope};

use app::errorwrap::Error;
use app::core::func::{save, REMOVE, SavePosition};
use app::core::image_merging::{merge_all_img_to_gfx, overlap_in_images};

fn wrapper_save(file_name: &str, str_add: &str, str_search: &str, str_above: bool, str_bottom: bool, mod_tag: &str, mod_install_state: u8) -> String {
	let str_position = match str_bottom {
		true => SavePosition::BOTTOM,
		false => match str_above {
			true => SavePosition::ABOVE,
			false => SavePosition::BELOW,
		},
	};

	let result = save(file_name, str_add, str_search, str_position, mod_tag, mod_install_state);
	match result {
		Ok(()) => "".to_string(),
		Err(e) => format!("(save) {}", e),
	}
}

fn wrapper_merge(mod_gfx: &str, index_file: &str, mod_tag: &str, mod_install_state: u8) -> String {
	if mod_install_state != REMOVE {
		const GFX_DIR: &str = "./gfx";
		let result = merge_all_img_to_gfx(mod_gfx, GFX_DIR, index_file, mod_tag);
		match result {
			Ok(_) => "".to_string(),
			Err(e) => format!("{}", e),
		}
	} else {
		"".to_string()
	}
}

fn wrapper_overlap(original_img: &str, on_top_img: &str, mod_install_state: u8) -> String {
	if mod_install_state != REMOVE {
		let result = overlap_in_images(original_img, on_top_img);
		match result {
			Ok(_) => "".to_string(),
			Err(e) => format!("{}", e),
		}
	} else {
		"".to_string()
	}
}


macro_rules! throw_on_err {
	($s:expr) => {
		concat!("RESULT = ", $s, " if RESULT != \"\" {throw RESULT;}")
	}
}

#[tauri::command]
pub fn load_mod(mod_file: &str, mod_tag: &str, mod_install_state: u8) -> Result<(), Error>{
	// Create a new person

	// Create a new Rhai Engine and Scope
	let mut engine = Engine::new();
	let mut scope = Scope::new();
	// Need scope.rewind() if you intend to make this global (reusable between mods)
	// https://rhai.rs/book/engine/scope.html#admonition-dont-forget-to-rewind

	// Register the Functions with the Rhai Engine
	engine
		.set_strict_variables(true)
		.set_allow_looping(false)
		.disable_symbol("let")
		.disable_symbol("const")
		.register_fn("Savefile", wrapper_save)
		.register_fn("Mergefile", wrapper_merge)
		.register_fn("Overlapfile", wrapper_overlap)
	;


	// Add the person object to the Rhai scope

	scope.push("File", "");
	scope.push("Add", "");
	scope.push("Search", "");
	scope.push("Above", false);
	scope.push("Bottom", false);

	scope.push("GfxFolder", "");
	scope.push("IndexFile", "");

	scope.push("RESULT", "");

	scope.push_constant("mod_tag", mod_tag.to_string());
	scope.push_constant("mod_install_state", mod_install_state);

	// Read the script
	println!("{:?}", std::env::current_dir()?.display());
	let mut script = std::fs::read_to_string(mod_file)?;
	script = script.replace("Save_This.", throw_on_err!("Savefile(File, Add, Search, Above, Bottom, mod_tag, mod_install_state);") );
	script = script.replace("Merge_This.", throw_on_err!("Mergefile(GfxFolder, IndexFile, mod_tag, mod_install_state);") );
	script = script.replace("Overlap_This.", throw_on_err!("Overlapfile(File, Add, mod_install_state);") );

	script.push_str("RESULT");

	// Evaluate a Rhai script
	let reslut = engine.eval_with_scope::<String>(&mut scope, &script);

	match reslut {
		Ok(_) => Ok(()),	// Code error
		Err(e) => Err(Error::ModScriptError(format!("{}", e))),							// Script parsing error
	}
}
