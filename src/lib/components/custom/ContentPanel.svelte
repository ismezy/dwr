<script lang="ts">
	import { projectsStore } from '$lib/stores/projects.svelte';
	import { ScrollArea } from '$lib/components/ui/scroll-area/index.js';
	import { FileText } from '@lucide/svelte';
</script>

<div class="flex flex-col h-full flex-1 bg-background">
	<div class="flex items-center gap-2 px-4 py-3 border-b shrink-0">
		<FileText class="h-4 w-4 text-muted-foreground" />
		<span class="text-sm font-semibold">内容</span>
	</div>

	{#if projectsStore.selected}
		<ScrollArea class="flex-1">
			<div class="p-6">
				<div class="text-2xl font-bold mb-2">{projectsStore.selected.name}</div>
				{#if projectsStore.selected.code}
					<div class="text-sm text-muted-foreground mb-4">编号: {projectsStore.selected.code}</div>
				{/if}
				<div class="text-sm text-muted-foreground mb-6">路径: {projectsStore.selected.path}</div>

				<div class="rounded-lg border bg-card p-6">
					<div class="text-sm text-muted-foreground text-center">
						日报内容将在这里展示
					</div>
				</div>
			</div>
		</ScrollArea>
	{:else}
		<div class="flex-1 flex items-center justify-center">
			<div class="text-sm text-muted-foreground text-center px-4">
				从左侧选择一个项目以查看详情
			</div>
		</div>
	{/if}
</div>
