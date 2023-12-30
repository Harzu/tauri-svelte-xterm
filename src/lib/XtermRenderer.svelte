<script>
    import { invoke } from '@tauri-apps/api/tauri';
    import { listen } from '@tauri-apps/api/event';
    import { onMount } from 'svelte'
    import { Terminal } from "xterm";
    import { FitAddon } from "xterm-addon-fit";
    import "../../node_modules/xterm/css/xterm.css";

    let term
    const fitAddon = new FitAddon()
    let resizeTimeout

    const resize = async (_) => {
        clearTimeout(resizeTimeout)

        setTimeout(async () => {
            fitAddon.fit()
            await invoke("resize_term", {
                rows: term.rows, cols: term.cols
            })
        }, 500)
    }

    onMount(async () => {
        term = new Terminal({
            fontFamily: `'Fira Mono', monospace`,
            fontSize: 15,
            fontWeight: 900,
            theme: {
                background: "#222222",
            }
        })

        term.loadAddon(fitAddon)
        term.open(document.querySelector("#terminal"))
        resize()

        term.onKey(async ({ key }) => {
            console.log(key)
            await invoke("write_to_pty", {data: key.charCodeAt(0)})
        })

        listen("term_data", (e) => {
            for (const cell of e.payload) {
                term.write(String.fromCharCode(cell))
            }
        })

        await invoke("init_raw_read")
    })
</script>

<div class="terminal-container">
    <div id="terminal"></div>
</div>
<svelte:window on:resize={resize} />

<style>
.terminal-container {
  /* this is important */
  overflow: hidden;
}

.terminal-container, #terminal {
    height: 99vh;
}

.xterm .xterm-viewport {
  /* see : https://github.com/xtermjs/xterm.js/issues/3564#issuecomment-1004417440 */
  width: initial !important;
}
</style>
