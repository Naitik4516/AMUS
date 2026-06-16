<script>
    import { Play } from "@lucide/svelte";
    import Button from "../ui/Button.svelte";
    import { FolderInput } from "@lucide/svelte";
    import { importAudioLibrary } from "$lib/commands.svelte";

    let { hasMusic } = $props();
</script>

<div
    class="relative w-full h-90 max-h-3/5 md:min-h-120 md:h-full rounded-[3rem] overflow-hidden group isolate"
>
    <!-- Background Gradient -->
    <div
        class="absolute inset-0 bg-linear-to-br from-secondary/60 via-secondary/20 to-dark transition-transform duration-700 group-hover:scale-110"
    ></div>
    <div
        class="absolute inset-0 bg-linear-to-r from-dark/90 via-dark/40 to-transparent mix-blend-multiply"
    ></div>
    <div
        class="absolute inset-0 bg-linear-to-t from-dark/80 via-transparent to-transparent"
    ></div>

    <!-- Decorative 3D Model -->
    <div
        class="absolute -right-8 md:right-0 top-1/2 -translate-y-1/2 w-[70%] sm:w-[50%] lg:w-[50%] h-[120%] pointer-events-none transition-transform duration-700 group-hover:scale-105 group-hover:-translate-x-4 group-hover:-rotate-3 z-0 opacity-90"
    >
        <div class="relative w-full h-full">
            <!-- Base Image with dynamic CSS hue rotation for the theme -->
            <img
                src="/headphones.png"
                alt="3D Headphones"
                class="w-full h-full object-contain object-right"
                style="filter: hue-rotate(var(--model-hue-rotate, 0deg))"
            />
        </div>
    </div>

    <!-- Content -->
    <div
        class="absolute inset-0 p-8 md:p-12 flex flex-col justify-end w-full md:w-2/3 lg:w-1/2 z-10"
    >
        <div class="animate-fade-in-up">
            <span
                class="inline-block px-3 py-1 bg-white/10 backdrop-blur-md rounded-full text-xs font-bold tracking-wider text-light mb-4 border border-white/5 uppercase"
            >
                Local Library
            </span>

            <h1
                class="text-3xl md:text-6xl font-black text-light mb-4 tracking-tight drop-shadow-2xl"
            >
                {hasMusic ? "Your Music, Your Way" : "Let's Fill the Silence"}
            </h1>
            <p
                class="text md:text-2xl text-light/80 font-medium mb-8 drop-shadow-md"
            >
                {hasMusic
                    ? "Rediscover your music collection with Amus - your personal jukebox for all your favorite tunes."
                    : "Dive into your personal collection and rediscover your favorite tunes with Amus."}
            </p>

            <div class="flex items-center gap-4">
                {#if hasMusic}
                    <button
                        class="bg-secondary text-dark font-bold p-4 md:px-8 md:py-4 rounded-full flex items-center gap-3 shadow-[0_0_40px_rgba(var(--color-secondary),0.4)] hover:scale-105 transition-transform cursor-pointer"
                    >
                        <Play class="w-5 h-5 fill-current" />
                        Shuffle All
                    </button>
                    <button
                        class="bg-white/10 hover:bg-white/20 backdrop-blur-md text-light border border-white/10 font-bold p-4 md:px-8 md:py-4 rounded-full flex items-center gap-3 transition-colors cursor-pointer"
                    >
                        <svg
                            xmlns="http://www.w3.org/2000/svg"
                            width="20"
                            height="20"
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke="currentColor"
                            stroke-width="2"
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            ><path d="M5 12h14" /><path d="M12 5v14" /></svg
                        >
                        Add Track
                    </button>
                {:else}
                    <Button text="Add Your Music" onClick={importAudioLibrary}>
                        <FolderInput size={20} />
                    </Button>
                {/if}
            </div>
        </div>
    </div>
</div>
