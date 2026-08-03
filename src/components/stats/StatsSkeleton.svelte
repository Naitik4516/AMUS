<div class="animate-pulse" aria-hidden="true">
    <!-- Overview cards -->
    <section class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-6 gap-4 mb-8">
        {#each Array(6) as _, i (i)}
            <div
                class="bg-card/50 border border-border rounded-3xl shadow-lg p-4 flex flex-col items-center gap-3"
            >
                <div class="size-8 rounded-full bg-neutral-800"></div>
                <div class="h-6 w-18 rounded-lg bg-neutral-800"></div>
                <div class="h-3.5 w-16 rounded bg-neutral-800/70"></div>
            </div>
        {/each}
    </section>

    <!-- Second row -->
    <section class="grid grid-cols-2 sm:grid-cols-4 gap-4 mb-8">
        {#each Array(4) as _, i (i)}
            <div
                class="bg-card/50 border border-border rounded-3xl shadow-lg p-4 flex flex-col items-center gap-3"
            >
                <div class="h-6 w-18 rounded-lg bg-neutral-800"></div>
                <div class="h-3.5 w-16 rounded bg-neutral-800/70"></div>
            </div>
        {/each}
    </section>

    <!-- Quality row -->
    <section class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-5 gap-4 mb-8">
        {#each Array(5) as _, i (i)}
            <div
                class="bg-card/50 border border-border rounded-3xl shadow-lg p-4 flex flex-col items-center gap-3"
            >
                <div class="h-6 w-18 rounded-lg bg-neutral-800"></div>
                <div class="h-3.5 w-16 rounded bg-neutral-800/70"></div>
            </div>
        {/each}
    </section>

    <div class="grid lg:grid-cols-3 gap-6">
        <!-- Left column -->
        <div class="lg:col-span-2 space-y-6">
            <div
                class="bg-card/50 border border-border rounded-3xl shadow-lg p-5 h-56"
            >
                <div class="h-5 w-40 rounded-lg bg-neutral-800 mb-6"></div>
                <div class="h-36 rounded-2xl bg-neutral-800/60"></div>
            </div>
            {#each Array(2) as _, i (i)}
                <div
                    class="bg-card/50 border border-border rounded-3xl shadow-lg p-5 h-44"
                >
                    <div class="h-5 w-44 rounded-lg bg-neutral-800 mb-6"></div>
                    <div class="h-28 rounded-2xl bg-neutral-800/60"></div>
                </div>
            {/each}
            {#each Array(2) as _, i (i)}
                <div
                    class="bg-card/50 border border-border rounded-3xl shadow-lg p-5 h-36"
                >
                    <div class="h-5 w-32 rounded-lg bg-neutral-800 mb-4"></div>
                    <div class="grid grid-cols-12 gap-1.5">
                        {#each Array(24) as _, j (j)}
                            <div class="size-5 rounded bg-neutral-800/60"></div>
                        {/each}
                    </div>
                </div>
            {/each}
        </div>

        <!-- Right column -->
        <div class="space-y-6">
            {#each Array(4) as _, i (i)}
                <div
                    class="bg-card/50 border border-border rounded-3xl shadow-lg p-5"
                >
                    <div class="h-5 w-32 rounded-lg bg-neutral-800 mb-4"></div>
                    <div class="space-y-3">
                        {#each Array(5) as _, j (j)}
                            <div class="flex items-center gap-3">
                                <div
                                    class="size-10 rounded-lg bg-neutral-800"
                                ></div>
                                <div class="flex-1 space-y-1.5">
                                    <div
                                        class="h-3.5 w-2/3 rounded bg-neutral-800"
                                    ></div>
                                    <div
                                        class="h-3 w-1/3 rounded bg-neutral-800/70"
                                    ></div>
                                </div>
                                <div
                                    class="h-3.5 w-10 rounded bg-neutral-800/70"
                                ></div>
                            </div>
                        {/each}
                    </div>
                </div>
            {/each}
        </div>
    </div>
</div>
