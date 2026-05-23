<script lang="ts">
	import type { DeviceInfo } from "$lib/DeviceInfo";
	import type { BackgroundMode } from "$lib/Profile";

	import { invoke } from "@tauri-apps/api/core";
	import { message } from "@tauri-apps/plugin-dialog";

	export let device: DeviceInfo;
	export let profileId: string;

	let image: string | null = null;
	let color: string = "#000000";
	let mode: BackgroundMode = "stretch";
	let loading = true;
	let saving = false;

	async function loadBackground() {
		loading = true;
		try {
			const [img, col, m] = await invoke<[string | null, string | null, string]>("get_background", {
				device: device.id,
				profile: profileId,
			});
			image = img || null;
			color = col || "#000000";
			mode = (m as BackgroundMode) || "stretch";
		} catch (error: any) {
			console.error("Failed to load background:", error);
		} finally {
			loading = false;
		}
	}

	$: if (device && profileId) {
		loadBackground();
	}

	async function saveBackground() {
		saving = true;
		try {
			await invoke("set_background", {
				device: device.id,
				profile: profileId,
				image,
				color: color || null,
				mode,
			});
		} catch (error: any) {
			message(error, { title: "Failed to save background" });
			console.error(error);
		} finally {
			saving = false;
		}
	}

	function handleFileDrop(event: DragEvent) {
		event.preventDefault();
		const file = event.dataTransfer?.files[0];
		if (file) readFile(file);
	}

	function handleFileSelect(event: Event) {
		const target = event.target as HTMLInputElement;
		const file = target.files?.[0];
		if (file) readFile(file);
	}

	function readFile(file: File) {
		if (!file.type.startsWith("image/")) {
			message("Please select an image file.", { title: "Invalid file" });
			return;
		}
		const reader = new FileReader();
		reader.onload = () => {
			image = reader.result as string;
		};
		reader.readAsDataURL(file);
	}

	function clearImage() {
		image = null;
	}
</script>

<div class="flex flex-col space-y-4 text-neutral-300">
	{#if loading}
		<div class="text-center py-8 text-neutral-400">Loading...</div>
	{:else}
		<!-- Preview -->
		<div
			class="relative w-full h-48 rounded-lg border border-neutral-600 overflow-hidden flex items-center justify-center"
			style:background-color={color}
			on:dragover|preventDefault
			on:drop={handleFileDrop}
		>
			{#if image}
				<img
					src={image}
					alt="Background preview"
					class="w-full h-full"
					class:object-contain={mode === "center"}
					class:object-cover={mode === "cover" || mode === "stretch"}
					class:object-fill={mode === "stretch"}
					class:object-none={mode === "tile"}
				/>
			{:else}
				<span class="text-neutral-500 text-sm">No background image</span>
			{/if}
		</div>

		<!-- Image Upload -->
		<div class="flex flex-row items-center space-x-2">
			<label
				class="grow px-4 py-2 text-center bg-neutral-700 hover:bg-neutral-600 transition-colors border border-neutral-600 rounded-lg cursor-pointer"
				on:dragover|preventDefault
				on:drop={handleFileDrop}
			>
				<input type="file" accept="image/*" class="hidden" on:change={handleFileSelect} />
				Click or drop image here
			</label>
			{#if image}
				<button
					class="px-4 py-2 bg-red-900 hover:bg-red-800 transition-colors border border-red-700 rounded-lg"
					on:click={clearImage}
				>
					Clear
				</button>
			{/if}
		</div>

		<!-- Color Picker -->
		<div class="flex flex-row items-center space-x-3">
			<label class="text-sm text-neutral-400 w-20">Color</label>
			<input
				type="color"
				bind:value={color}
				class="w-12 h-10 bg-transparent border-0 cursor-pointer"
			/>
			<input
				type="text"
				bind:value={color}
				class="grow p-2 bg-neutral-700 border border-neutral-600 rounded-lg text-neutral-300 font-mono text-sm"
				pattern="^#[0-9A-Fa-f]{6}$"
			/>
		</div>

		<!-- Mode -->
		<div class="flex flex-row items-center space-x-3">
			<label class="text-sm text-neutral-400 w-20">Mode</label>
			<select
				bind:value={mode}
				class="grow p-2 bg-neutral-700 border border-neutral-600 rounded-lg text-neutral-300"
			>
				<option value="stretch">Stretch</option>
				<option value="tile">Tile</option>
				<option value="center">Center</option>
				<option value="cover">Cover</option>
			</select>
		</div>

		<!-- Save -->
		<button
			class="px-4 py-2 bg-green-800 hover:bg-green-700 transition-colors border border-green-700 rounded-lg font-semibold disabled:opacity-50"
			on:click={saveBackground}
			disabled={saving}
		>
			{saving ? "Saving..." : "Save Background"}
		</button>
	{/if}
</div>
