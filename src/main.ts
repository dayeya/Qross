import { browse_file, compress} from "./frontend_logic/functionality";

export let log: HTMLElement | null;
export let img: HTMLImageElement | null;

// Buttons
let browse_btn: HTMLElement | null;
let compress_btn: HTMLElement | null;
let decompress_bth: HTMLElement | null;
let analyz_btn: HTMLElement | null;

window.addEventListener("DOMContentLoaded", () => {
  const querySelector = (id: string) => document.querySelector(id) as HTMLElement ;

  browse_btn = querySelector("#browse_btn")
  compress_btn = querySelector("#compress_btn")
  decompress_bth = querySelector("#decompress_btn")
  analyz_btn = querySelector("#analyze_btn")

  log = querySelector("#result")
  img = document.querySelector("#image")

  browse_btn.addEventListener("click", (e: Event) => {
    e.preventDefault();

    // Browse functionality
    browse_file();
  });

  compress_btn.addEventListener("click", (e: Event) => {
    e.preventDefault();

    // Browse functionality
    compress();
  });
});