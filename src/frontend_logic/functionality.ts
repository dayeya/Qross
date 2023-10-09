import { invoke } from "@tauri-apps/api/tauri";
import { open } from "@tauri-apps/api/dialog";
import { convertFileSrc } from '@tauri-apps/api/tauri';
import { log, img } from "../main";


const FILE_DIALOG_ARGS = {
  multiple: false,
  title: 'Select a file'
}; 

export async function browse_file() {

    console.log("Browsing! :>");
    const selected_path = await open(FILE_DIALOG_ARGS) as string;
    const final_path: string = await invoke("save_file_inside_db", { file: selected_path });

    // Update GUI.
    console.log(final_path)
    if(log && img) {
      img.src = convertFileSrc(final_path);
    }
}

export async function compress() {
    const saved_bits = await invoke("compress");
    console.log(saved_bits)
}
