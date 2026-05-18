<script lang="ts">
	import { projectsStore } from '$lib/stores/projects.svelte';
	import { ScrollArea } from '$lib/components/ui/scroll-area/index.js';
	import { CalendarDays, FileText } from '@lucide/svelte';
	import { cn } from '$lib/utils';

	// 占位：日报列表
	let selectedDate = $state<string | null>(null);

	const placeholderDates = [
		{ date: '2026-05-18', label: '今天' },
		{ date: '2026-05-17', label: '昨天' },
		{ date: '2026-05-16', label: '周五' },
		{ date: '2026-05-15', label: '周四' },
		{ date: '2026-05-14', label: '周三' },
	];
</script>

<div class="flex flex-col h-full w-56 border-r bg-muted/30">
	<div class="flex items-center gap-2 px-3 py-3 border-b shrink-0">
		<CalendarDays class="h-4 w-4 text-muted-foreground" />
		<span class="text-sm font-semibold">每日日报</span>
	</div>

	{#if projectsStore.selected}
		<ScrollArea class="flex-1">
			<div class="p-2 space-y-1">
				{#each placeholderDates as item (item.date)}
					<button
						class={cn(
							'w-full text-left rounded-md px-3 py-2 transition-colors flex items-center gap-2',
							selectedDate === item.date
								? 'bg-accent text-accent-foreground'
								: 'hover:bg-accent/50 text-foreground'
						)}
						onclick={() => (selectedDate = item.date)}
					>
						<FileText class="h-4 w-4 shrink-0 opacity-60" />
						<div class="flex-1 min-w-0">
							<div class="text-sm font-medium">{item.date}</div>
							<div class="text-xs text-muted-foreground">{item.label}</div>
						</div>
					</button>
				{/each}
			</div>
		</ScrollArea>
	{:else}
		<div class="flex-1 flex items-center justify-center">
			<div class="text-sm text-muted-foreground text-center px-4">
				选择一个项目以查看日报
			</div>
		</div>
	{/if}
</div>
