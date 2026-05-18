<script lang="ts">
	import { projectsStore } from '$lib/stores/projects.svelte';
	import { configStore } from '$lib/stores/config.svelte';
	import { reportsStore } from '$lib/stores/reports.svelte';
	import { i18n } from '$lib/i18n';
	import { ScrollArea } from '$lib/components/ui/scroll-area/index.js';
	import { FileText, User, Folder, Hash } from '@lucide/svelte';
</script>

<div class="flex flex-col h-full w-full bg-background">
	<div class="flex items-center gap-2 px-4 py-3 border-b shrink-0">
		<FileText class="h-4 w-4 text-muted-foreground" />
		<span class="text-sm font-semibold">{i18n.t('content.title')}</span>
	</div>

	{#if projectsStore.selected}
		<ScrollArea class="flex-1">
			{#if reportsStore.selectedDate && reportsStore.content}
				<div class="p-6">
					<div class="text-2xl font-bold mb-4">{projectsStore.selected.name}</div>
					<div class="prose dark:prose-invert max-w-none">
						<pre class="whitespace-pre-wrap font-mono text-sm leading-relaxed">{reportsStore.content}</pre>
					</div>
				</div>
			{:else}
				<div class="p-6">
					<div class="text-2xl font-bold mb-2">{projectsStore.selected.name}</div>

					<div class="space-y-3 text-sm text-muted-foreground mb-6">
						{#if projectsStore.selected.code}
							<div class="flex items-center gap-2">
								<Hash class="h-4 w-4" />
								<span>{projectsStore.selected.code}</span>
							</div>
						{/if}
						<div class="flex items-center gap-2">
							<Folder class="h-4 w-4" />
							<span class="truncate">{projectsStore.selected.path}</span>
						</div>
						<div class="flex items-center gap-2">
							<User class="h-4 w-4" />
							<span>
								{i18n.t('config.gitUserName')}:
								{#if projectsStore.selected.git_user_name}
									{projectsStore.selected.git_user_name}
									<span class="text-xs text-muted-foreground/60">({i18n.t('content.gitUser.project')})</span>
								{:else if configStore.configs.git_user_name}
									{configStore.configs.git_user_name}
									<span class="text-xs text-muted-foreground/60">({i18n.t('content.gitUser.global')})</span>
								{:else}
									<span class="text-xs text-destructive">{i18n.t('content.gitUser.unconfigured')}</span>
								{/if}
							</span>
						</div>
					</div>

					<div class="rounded-lg border bg-card p-6">
						<div class="text-sm text-muted-foreground text-center">
							{i18n.t('content.placeholder')}
						</div>
					</div>
				</div>
			{/if}
		</ScrollArea>
	{:else}
		<div class="flex-1 flex items-center justify-center">
			<div class="text-sm text-muted-foreground text-center px-4">
				{i18n.t('content.emptyHint')}
			</div>
		</div>
	{/if}
</div>
