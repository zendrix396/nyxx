<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { onMount } from 'svelte';

  type MacroHttpMethod = 'GET' | 'POST';
  /** Must match Rust `MacroDataKind` serde: lowercase none | text | image */
  type MacroDataKind = 'none' | 'text' | 'image';

  interface MacroProfile {
    name: string;
    method: MacroHttpMethod;
    input_kind: MacroDataKind;
    output_kind: MacroDataKind;
    description: string;
    use_when?: string;
  }

  interface MacroConfig {
    name: string;
    profile: MacroProfile;
  }

  interface InvokeResponse {
    success: boolean;
    macro_name: string;
    message: string;
    output_text?: string | null;
    output_image_base64?: string | null;
  }

  let appState = 'IDLE';
  let status = '';
  let macros: MacroConfig[] = [];

  let recordName = '';
  let recordMethod: MacroHttpMethod = 'GET';
  let recordInputKind: MacroDataKind = 'none';
  let recordOutputKind: MacroDataKind = 'none';
  let recordDescription = '';
  let recordUseWhen = '';

  let testMacroName = '';
  let testPayloadText = '';
  let testResponse = '';
  let imagePreview = '';

  async function refreshMacros() {
    try {
      macros = await invoke<MacroConfig[]>('list_macro_configs_command');
      if (!testMacroName && macros.length > 0) {
        testMacroName = macros[0].name;
      }
    } catch (error) {
      status = `Failed to load macros: ${error}`;
    }
  }

  async function startRecording() {
    try {
      await invoke('start_recording_command');
      status = 'Recording started.';
    } catch (error) {
      status = `Start recording failed: ${error}`;
    }
  }

  async function stopRecording() {
    if (!recordName.trim()) {
      status = 'Macro name is required.';
      return;
    }

    try {
      const name = recordName.trim();
      await invoke('stop_recording_command', { name });
      await invoke('set_macro_profile_command', {
        name,
        method: recordMethod,
        inputKind: recordInputKind,
        outputKind: recordOutputKind,
        description: recordDescription.trim() || `${name} macro`,
        useWhen: recordUseWhen.trim() || ''
      });
      status = `Macro '${name}' saved with profile.`;
      await refreshMacros();
      testMacroName = name;
      recordName = '';
      recordUseWhen = '';
    } catch (error) {
      status = `Stop recording failed: ${error}`;
    }
  }

  async function deleteMacro(name: string) {
    try {
      await invoke('delete_macro_command', { name });
      status = `Macro '${name}' deleted.`;
      await refreshMacros();
      if (testMacroName === name) {
        testMacroName = macros[0]?.name ?? '';
      }
    } catch (error) {
      status = `Delete failed: ${error}`;
    }
  }

  async function stopRunningMacro() {
    try {
      const msg = await invoke<string>('stop_running_macro_command');
      status = msg;
    } catch (error) {
      status = `Stop macro failed: ${error}`;
    }
  }

  async function invokeMacro() {
    imagePreview = '';
    testResponse = '';
    const name = testMacroName.trim();
    if (!name) {
      status = 'Select a macro first.';
      return;
    }

    const selected = macros.find((item) => item.name === name);
    if (!selected) {
      status = 'Selected macro profile not found.';
      return;
    }

    try {
      let response: Response;
      if (selected.profile.method === 'POST') {
        response = await fetch(`http://127.0.0.1:4777/macros/${encodeURIComponent(name)}/invoke`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ text: testPayloadText || null })
        });
      } else {
        const url = new URL(`http://127.0.0.1:4777/macros/${encodeURIComponent(name)}/invoke`);
        if (testPayloadText.trim()) {
          url.searchParams.set('input', testPayloadText);
        }
        response = await fetch(url.toString(), { method: 'GET' });
      }

      const data = (await response.json()) as InvokeResponse | { message?: string };
      if (!response.ok) {
        status = `Invoke failed: ${(data as { message?: string }).message ?? response.statusText}`;
        return;
      }

      const invokeData = data as InvokeResponse;
      status = invokeData.message;
      if (invokeData.output_text) {
        testResponse = invokeData.output_text;
      }
      if (invokeData.output_image_base64) {
        imagePreview = `data:image/png;base64,${invokeData.output_image_base64}`;
      }
    } catch (error) {
      status = `Invoke request failed: ${error}`;
    }
  }

  onMount(() => {
    refreshMacros();
    const unlistenStateChanged = listen('app_state_changed', (event) => {
      appState = String(event.payload);
    });

    return () => {
      unlistenStateChanged.then((fn) => fn());
    };
  });
</script>

<main class="container">
  <h1>Nyx Macro Recorder</h1>
  <p class="sub">State: {appState}</p>

  <section class="card">
    <h2>Record Macro</h2>
    <div class="grid">
      <input bind:value={recordName} placeholder="Macro name (e.g. img-gen)" />
      <select bind:value={recordMethod}>
        <option value="GET">GET</option>
        <option value="POST">POST</option>
      </select>
      <select bind:value={recordInputKind}>
        <option value="none">Input: None</option>
        <option value="text">Input: Text</option>
        <option value="image">Input: Image</option>
      </select>
      <select bind:value={recordOutputKind}>
        <option value="none">Output: None</option>
        <option value="text">Output: Text</option>
        <option value="image">Output: Image</option>
      </select>
      <input bind:value={recordDescription} placeholder="Description" />
      <input bind:value={recordUseWhen} placeholder="Use when..." />
    </div>
    <div class="row">
      <button on:click={startRecording} disabled={appState === 'RECORDING'}>Start Recording</button>
      <button on:click={stopRecording} disabled={appState !== 'RECORDING'}>Stop & Save</button>
      <button on:click={stopRunningMacro}>Stop Running Macro</button>
    </div>
  </section>

  <section class="card">
    <h2>Available Macros</h2>
    {#if macros.length === 0}
      <p>No macros available.</p>
    {:else}
      <ul>
        {#each macros as item (item.name)}
          <li>
            <div>
              <strong>{item.name}</strong>
              <span>{item.profile.method} | in: {item.profile.input_kind} | out: {item.profile.output_kind}</span>
              {#if item.profile.use_when}
                <span>use: {item.profile.use_when}</span>
              {/if}
            </div>
            <button class="danger" on:click={() => deleteMacro(item.name)}>Delete</button>
          </li>
        {/each}
      </ul>
    {/if}
  </section>

  <section class="card">
    <h2>Invoke Macro API (Localhost)</h2>
    <div class="grid">
      <select bind:value={testMacroName}>
        {#each macros as item (item.name)}
          <option value={item.name}>{item.name}</option>
        {/each}
      </select>
      <textarea bind:value={testPayloadText} rows="3" placeholder="Text payload for clipboard (POST or GET input query)"></textarea>
    </div>
    <div class="row">
      <button on:click={invokeMacro}>Invoke Selected Macro</button>
      <button on:click={refreshMacros}>Refresh</button>
    </div>
    {#if testResponse}
      <pre>{testResponse}</pre>
    {/if}
    {#if imagePreview}
      <img src={imagePreview} alt="Clipboard output" />
    {/if}
  </section>

  {#if status}
    <p class="status">{status}</p>
  {/if}
</main>

<style>
  :global(body) {
    margin: 0;
    font-family: system-ui, sans-serif;
    background: #111;
    color: #f5f5f5;
    overflow: auto;
  }
  :global(html), :global(body), :global(#app) { height: 100%; }
  .container {
    box-sizing: border-box;
    height: 100%;
    padding: 16px;
    display: grid;
    gap: 12px;
    overflow-y: auto;
    align-content: start;
  }
  .sub { opacity: 0.8; margin-top: -8px; }
  .card { border: 1px solid #333; border-radius: 10px; padding: 12px; display: grid; gap: 10px; background: #1a1a1a; }
  .grid { display: grid; gap: 8px; }
  .row { display: flex; gap: 8px; flex-wrap: wrap; }
  input, textarea, select, button { border-radius: 8px; border: 1px solid #444; padding: 8px; background: #0f0f0f; color: #f5f5f5; }
  button { cursor: pointer; }
  button.danger { border-color: #8b2f2f; color: #ffb0b0; }
  ul { list-style: none; margin: 0; padding: 0; display: grid; gap: 8px; }
  li { display: flex; justify-content: space-between; align-items: center; border: 1px solid #2b2b2b; border-radius: 8px; padding: 8px; gap: 8px; }
  li span { display: block; font-size: 12px; opacity: 0.75; margin-top: 2px; }
  pre { background: #101010; border: 1px solid #2f2f2f; border-radius: 8px; padding: 8px; white-space: pre-wrap; max-height: 220px; overflow: auto; }
  img { max-width: 100%; border: 1px solid #333; border-radius: 8px; }
  .status { border: 1px solid #2f7d4a; background: #173122; padding: 10px; border-radius: 8px; }
</style>
