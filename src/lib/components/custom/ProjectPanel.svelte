<script lang="ts">
	import { open } from '@tauri-apps/plugin-dialog';
	import { Button } from '$lib/components/ui/button/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import { Label } from '$lib/components/ui/label/index.js';
	import { ScrollArea } from '$lib/components/ui/scroll-area/index.js';
	import {
		Dialog,
		DialogContent,
		DialogHeader,
		DialogTitle,
		DialogFooter,
	} from '$lib/components/ui/dialog/index.js';
	import { projectsStore, type Project } from '$lib/stores/projects.svelte';
	import { i18n } from '$lib/i18n';
	import { Folder, Plus, Pencil, Trash2, ChevronLeft, ChevronRight, FolderGit2 } from '@lucide/svelte';
	import { cn } from '$lib/utils';

	function getInitialCollapsed(): boolean {
		try {
			return localStorage.getItem('dwr:project-panel-collapsed') === 'true';
		} catch {
			return false;
		}
	}

	let collapsed = $state(getInitialCollapsed());
	let dialogOpen = $state(false);

	$effect(() => {
		try {
			localStorage.setItem('dwr:project-panel-collapsed', String(collapsed));
		} catch {
			// noop: localStorage may not be available in release builds
		}
	});
	let editingId = $state<string | null>(null);
	let formName = $state('');
	let formCode = $state('');
	let formPath = $state('');
	let formGitUserName = $state('');

	function resetForm() {
		formName = '';
		formCode = '';
		formPath = '';
		formGitUserName = '';
		editingId = null;
	}

	function openAdd() {
		resetForm();
		dialogOpen = true;
	}

	function openEdit(project: Project) {
		editingId = project.id;
		formName = project.name;
		formCode = project.code ?? '';
		formPath = project.path;
		formGitUserName = project.git_user_name ?? '';
		dialogOpen = true;
	}

	async function pickFolder() {
		const selected = await open({
			directory: true,
			multiple: false,
		});
		if (selected && typeof selected === 'string') {
			formPath = selected;
		}
	}

	async function handleSave() {
		const name = formName.trim();
		const path = formPath.trim();
		if (!name || !path) return;

		if (editingId) {
			await projectsStore.update(editingId, {
				name,
				code: formCode.trim() || undefined,
				path,
				git_user_name: formGitUserName.trim() || undefined,
			});
		} else {
			await projectsStore.add({
				name,
				code: formCode.trim() || undefined,
				path,
				git_user_name: formGitUserName.trim() || undefined,
			});
		}
		dialogOpen = false;
		resetForm();
	}

	async function handleDelete(id: string, e: Event) {
		e.stopPropagation();
		await projectsStore.remove(id);
	}
</script>

<div
	class={cn(
		'flex flex-col h-full bg-sidebar transition-all duration-300 shrink-0',
		collapsed ? 'w-12' : 'w-64'
	)}
>
	<!-- Header -->
	<div class="flex items-center justify-between px-2 py-3 border-b shrink-0">
		{#if !collapsed}
			<span class="text-sm font-semibold text-sidebar-foreground truncate">{i18n.t('project.title')}</span>
		{/if}
		<div class="flex items-center gap-1">
			{#if !collapsed}
				<Button variant="ghost" size="icon" class="h-7 w-7" onclick={openAdd} title={i18n.t('project.add')}>
					<Plus class="h-4 w-4" />
				</Button>
			{/if}
			<Button
				variant="ghost"
				size="icon"
				class="h-7 w-7"
				onclick={() => (collapsed = !collapsed)}
				title={collapsed ? i18n.t('project.expand') : i18n.t('project.collapse')}
			>
				{#if collapsed}
					<ChevronRight class="h-4 w-4" />
				{:else}
					<ChevronLeft class="h-4 w-4" />
				{/if}
			</Button>
		</div>
	</div>

	<!-- List -->
	{#if !collapsed}
		<ScrollArea class="flex-1">
			<div class="p-2 space-y-1">
				{#each projectsStore.projects as project (project.id)}
					<button
						class={cn(
							'w-full text-left rounded-md px-2 py-2 group transition-colors',
							projectsStore.selectedId === project.id
								? 'bg-sidebar-accent text-sidebar-accent-foreground'
								: 'hover:bg-sidebar-accent/50 text-sidebar-foreground'
						)}
						onclick={() => projectsStore.select(project.id)}
					>
						<div class="flex items-center gap-2">
							<FolderGit2 class="h-4 w-4 shrink-0 opacity-70" />
							<div class="flex-1 min-w-0">
								<div class="text-sm font-medium truncate">{project.name}</div>
								{#if project.code}
									<div class="text-xs text-muted-foreground truncate">{project.code}</div>
								{/if}
							</div>
							<div class="flex items-center gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity">
								<Button
									variant="ghost"
									size="icon"
									class="h-6 w-6"
									onclick={(e) => {
										e.stopPropagation();
										openEdit(project);
									}}
									title={i18n.t('project.edit')}
								>
									<Pencil class="h-3 w-3" />
								</Button>
								<Button
									variant="ghost"
									size="icon"
									class="h-6 w-6 text-destructive hover:text-destructive"
									onclick={(e) => handleDelete(project.id, e)}
									title={i18n.t('common.delete')}
								>
									<Trash2 class="h-3 w-3" />
								</Button>
							</div>
						</div>
					</button>
				{:else}
					<div class="text-xs text-muted-foreground text-center py-8 px-2">
						{i18n.t('project.emptyHint')}
					</div>
				{/each}
			</div>
		</ScrollArea>
	{:else}
		<div class="flex-1 flex flex-col items-center pt-2 gap-1">
			<Button variant="ghost" size="icon" class="h-8 w-8" onclick={openAdd} title={i18n.t('project.add')}>
				<Plus class="h-4 w-4" />
			</Button>
			{#each projectsStore.projects as project (project.id)}
				<button
					class={cn(
						'w-8 h-8 flex items-center justify-center rounded-md transition-colors',
						projectsStore.selectedId === project.id
							? 'bg-sidebar-accent text-sidebar-accent-foreground'
							: 'hover:bg-sidebar-accent/50 text-sidebar-foreground'
					)}
					onclick={() => projectsStore.select(project.id)}
					title={project.name}
				>
					<FolderGit2 class="h-4 w-4" />
				</button>
			{/each}
		</div>
	{/if}
</div>

<!-- Add/Edit Dialog -->
<Dialog bind:open={dialogOpen} onOpenChange={(v) => !v && resetForm()}>
	<DialogContent class="sm:max-w-md">
		<DialogHeader>
			<DialogTitle>{editingId ? i18n.t('project.edit') : i18n.t('project.add')}</DialogTitle>
		</DialogHeader>
		<div class="grid gap-4 py-2">
			<div class="grid gap-2">
				<Label for="project-name">{i18n.t('project.name')} *</Label>
				<Input id="project-name" bind:value={formName} placeholder={i18n.t('project.name')} />
			</div>
			<div class="grid gap-2">
				<Label for="project-code">{i18n.t('project.code')}</Label>
				<Input id="project-code" bind:value={formCode} placeholder={i18n.t('common.optional')} />
			</div>
			<div class="grid gap-2">
				<Label for="project-path">{i18n.t('project.path')} *</Label>
				<div class="flex gap-2">
					<Input
						id="project-path"
						bind:value={formPath}
						placeholder={i18n.t('project.path')}
						class="flex-1"
						readonly
					/>
					<Button variant="outline" size="icon" onclick={pickFolder} title={i18n.t('common.selectFolder')}>
						<Folder class="h-4 w-4" />
					</Button>
				</div>
			</div>
			<div class="grid gap-2">
				<div class="flex items-center gap-2">
					<Label for="project-git-user" class="flex-1">{i18n.t('project.gitUserName')}</Label>
					<span class="text-xs text-muted-foreground">{i18n.t('project.gitUserNameHint')}</span>
				</div>
				<div class="flex gap-2">
					<Input
						id="project-git-user"
						bind:value={formGitUserName}
						placeholder={i18n.t('common.optional')}
						class="flex-1"
					/>
				</div>
			</div>
		</div>
		<DialogFooter>
			<Button variant="outline" onclick={() => { dialogOpen = false; resetForm(); }}>{i18n.t('common.cancel')}</Button>
			<Button onclick={handleSave} disabled={!formName.trim() || !formPath.trim()}>
				{i18n.t('common.save')}
			</Button>
		</DialogFooter>
	</DialogContent>
</Dialog>
