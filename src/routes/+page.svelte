<script lang="ts">
	import ProjectPanel from '$lib/components/custom/ProjectPanel.svelte';
	import DailyReportPanel from '$lib/components/custom/DailyReportPanel.svelte';
	import ContentPanel from '$lib/components/custom/ContentPanel.svelte';
	import ConfigDialog from '$lib/components/custom/ConfigDialog.svelte';
	import * as Resizable from '$lib/components/ui/resizable/index.js';
	import { i18n } from '$lib/i18n';
</script>

<div class="flex flex-col h-screen w-full overflow-hidden">
	<!-- Top bar -->
	<div class="flex items-center justify-between px-4 h-12 border-b shrink-0 bg-background">
		<div class="flex items-center gap-2">
			<span class="text-sm font-semibold">{i18n.t('app.name')}</span>
			<span class="text-xs text-muted-foreground">{i18n.t('app.subtitle')}</span>
		</div>
		<ConfigDialog />
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
