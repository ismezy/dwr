<script lang="ts">
	import ProjectPanel from '$lib/components/custom/ProjectPanel.svelte';
	import DailyReportPanel from '$lib/components/custom/DailyReportPanel.svelte';
	import ContentPanel from '$lib/components/custom/ContentPanel.svelte';
	import * as Resizable from '$lib/components/ui/resizable/index.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import { i18n } from '$lib/i18n';
	import { Settings } from '@lucide/svelte';
	import { goto } from '$app/navigation';
</script>

<div class="flex flex-col h-screen w-full overflow-hidden">
	<!-- Top bar -->
	<div class="flex items-center justify-between px-4 h-12 border-b shrink-0 bg-background">
		<div class="flex items-center gap-2">
			<span class="text-sm font-semibold">{i18n.t('app.name')}</span>
			<span class="text-xs text-muted-foreground">{i18n.t('app.subtitle')}</span>
		</div>
		<Button
			variant="ghost"
			size="icon"
			class="h-8 w-8"
			onclick={() => goto('/settings')}
			title={i18n.t('settings.title')}
		>
			<Settings class="h-4 w-4" />
		</Button>
	</div>

	<!-- Layout: fixed left sidebar + resizable right two columns -->
	<div class="flex flex-1 overflow-hidden">
		<ProjectPanel />

		<Resizable.ResizablePaneGroup direction="horizontal" class="flex-1">
			<Resizable.ResizablePane
				defaultSize={30}
				minSize={20}
				maxSize={50}
			>
				<DailyReportPanel />
			</Resizable.ResizablePane>

			<Resizable.ResizableHandle withHandle />

			<Resizable.ResizablePane
				defaultSize={70}
				minSize={30}
			>
				<ContentPanel />
			</Resizable.ResizablePane>
		</Resizable.ResizablePaneGroup>
	</div>
</div>
