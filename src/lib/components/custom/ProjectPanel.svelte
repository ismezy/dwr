<script lang="ts">
	import { open } from '@tauri-apps/plugin-dialog';
	import { invoke } from '@tauri-apps/api/core';
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
	import * as Select from '$lib/components/ui/select/index.js';
	import * as Popover from '$lib/components/ui/popover/index.js';
	import { projectsStore, type Project } from '$lib/stores/projects.svelte';
	import { toastStore } from '$lib/stores/toast.svelte';
	import { i18n } from '$lib/i18n';
	import { Folder, Plus, Pencil, Trash2, ChevronLeft, ChevronRight, ChevronDown, FolderGit2, FileText, FolderPlus, Check, ChevronsUpDown } from '@lucide/svelte';
	import { cn } from '$lib/utils';

	function getInitialCollapsed(): boolean {
		try {
			return localStorage.getItem('dwr:project-panel-collapsed') === 'true';
		} catch {
			return false;
		}
	}

	let collapsed = $state(getInitialCollapsed());

	$effect(() => {
		try {
			localStorage.setItem('dwr:project-panel-collapsed', String(collapsed));
		} catch {
			// noop: localStorage may not be available in release builds
		}
	});

	// 收起状态集合：不在集合中的项目默认展开
	let collapsedGroups = $state<Set<string>>(new Set());

	function toggleGroup(groupId: string) {
		const next = new Set(collapsedGroups);
		if (next.has(groupId)) {
			next.delete(groupId);
		} else {
			next.add(groupId);
		}
		collapsedGroups = next;
	}

	function isGroupExpanded(groupId: string): boolean {
		return !collapsedGroups.has(groupId);
	}

	// ---- 项目（第一级）表单 ----
	let projectDialogOpen = $state(false);
	let editingProjectId = $state<string | null>(null);
	let formName = $state('');
	let formCode = $state('');

	function resetProjectForm() {
		formName = '';
		formCode = '';
		editingProjectId = null;
	}

	function openAddProject() {
		resetProjectForm();
		projectDialogOpen = true;
	}

	function openEditProject(project: Project) {
		editingProjectId = project.id;
		formName = project.name;
		formCode = project.code ?? '';
		projectDialogOpen = true;
	}

	async function handleSaveProject() {
		const name = formName.trim();
		if (!name) return;
		if (editingProjectId) {
			await projectsStore.update(editingProjectId, {
				name,
				code: formCode.trim() || undefined,
			});
		} else {
			await projectsStore.add({
				name,
				code: formCode.trim() || undefined,
				path: '',
				project_type: 'code',
				parent_id: null,
			});
		}
		projectDialogOpen = false;
		resetProjectForm();
	}

	async function handleDeleteProject(project: Project, e: Event) {
		e.stopPropagation();
		if (projectsStore.dirsOf(project.id).length > 0) {
			toastStore.show(i18n.t('project.deleteHasDirs'), 'error');
			return;
		}
		await projectsStore.remove(project.id);
	}

	// ---- 项目目录（第二级）表单 ----
	let dirDialogOpen = $state(false);
	let editingDirId = $state<string | null>(null);
	let dirName = $state('');
	let dirPath = $state('');
	let dirGitUserName = $state('');
	let dirBranches = $state<string[]>([]);
	let availableBranches = $state<string[]>([]);
	let branchLoadFailed = $state(false);
	let branchPopoverOpen = $state(false);
	let branchFilter = $state('');
	let dirType = $state<'code' | 'docs'>('code');
	let dirParentId = $state<string>('');

	const typeItems = [
		{ value: 'code', labelKey: 'project.typeCode' },
		{ value: 'docs', labelKey: 'project.typeDocs' },
	] as const;
	let selectedTypeLabel = $derived(
		i18n.t(typeItems.find((t) => t.value === dirType)?.labelKey ?? 'project.typeCode')
	);
	let selectedParentLabel = $derived(
		projectsStore.groups.find((g) => g.id === dirParentId)?.name ?? ''
	);
	// 可选分支 = 仓库分支 + 已保存的分支（防止仓库读取失败或分支被删后丢配置）
	let branchOptions = $derived(
		[...new Set([...availableBranches, ...dirBranches])]
	);
	// 模糊过滤（大小写不敏感的子串匹配）
	let filteredBranches = $derived(
		branchFilter.trim()
			? branchOptions.filter((b) => b.toLowerCase().includes(branchFilter.trim().toLowerCase()))
			: branchOptions
	);

	function toggleBranch(branch: string) {
		dirBranches = dirBranches.includes(branch)
			? dirBranches.filter((b) => b !== branch)
			: [...dirBranches, branch];
	}

	async function loadBranches() {
		branchLoadFailed = false;
		const path = dirPath.trim();
		if (!path) {
			availableBranches = [];
			return;
		}
		try {
			availableBranches = await invoke<string[]>('list_branches', { projectPath: path });
		} catch (e) {
			console.error('failed to load branches:', e);
			availableBranches = [];
			branchLoadFailed = true;
		}
	}

	$effect(() => {
		if (dirDialogOpen && dirType === 'code' && dirPath) {
			loadBranches();
		}
	});

	function resetDirForm() {
		dirName = '';
		dirPath = '';
		dirGitUserName = '';
		dirBranches = [];
		availableBranches = [];
		branchLoadFailed = false;
		branchPopoverOpen = false;
		branchFilter = '';
		dirType = 'code';
		dirParentId = '';
		editingDirId = null;
	}

	function openAddDir(groupId: string, e?: Event) {
		e?.stopPropagation();
		resetDirForm();
		dirParentId = groupId;
		dirDialogOpen = true;
	}

	function openEditDir(dir: Project, e?: Event) {
		e?.stopPropagation();
		editingDirId = dir.id;
		dirName = dir.name;
		dirPath = dir.path;
		dirGitUserName = dir.git_user_name ?? '';
		dirBranches = dir.branch ? dir.branch.split(',').map((b) => b.trim()).filter(Boolean) : [];
		dirType = dir.project_type ?? 'code';
		dirParentId = dir.parent_id ?? '';
		dirDialogOpen = true;
	}

	async function pickFolder() {
		const selected = await open({
			directory: true,
			multiple: false,
		});
		if (selected && typeof selected === 'string') {
			dirPath = selected;
			if (!dirName.trim()) {
				const parts = selected.split(/[\\/]/).filter(Boolean);
				dirName = parts[parts.length - 1] ?? '';
			}
		}
	}

	async function handleSaveDir() {
		const name = dirName.trim();
		const path = dirPath.trim();
		if (!name || !path || !dirParentId) return;

		const branchValue = dirBranches.length > 0 ? dirBranches.join(', ') : undefined;
		if (editingDirId) {
			await projectsStore.update(editingDirId, {
				name,
				path,
				git_user_name: dirGitUserName.trim() || undefined,
				project_type: dirType,
				parent_id: dirParentId,
				branch: branchValue,
			});
		} else {
			await projectsStore.add({
				name,
				path,
				git_user_name: dirGitUserName.trim() || undefined,
				project_type: dirType,
				parent_id: dirParentId,
				branch: branchValue,
			});
		}
		dirDialogOpen = false;
		resetDirForm();
	}

	async function handleDeleteDir(id: string, e: Event) {
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
				<Button variant="ghost" size="icon" class="h-7 w-7" onclick={openAddProject} title={i18n.t('project.add')}>
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

	<!-- Tree -->
	{#if !collapsed}
		<ScrollArea class="flex-1">
			<div class="p-2 space-y-0.5">
				{#each projectsStore.groups as group (group.id)}
					<!-- 项目行 -->
					<div
						class={cn(
							'w-full rounded-md px-2 py-1.5 group transition-colors',
							projectsStore.selectedId === group.id
								? 'bg-sidebar-accent text-sidebar-accent-foreground'
								: 'hover:bg-sidebar-accent/50 text-sidebar-foreground'
						)}
					>
						<div class="flex items-center gap-1">
							<button
								class="flex items-center gap-1.5 flex-1 min-w-0 text-left"
								onclick={() => { projectsStore.select(group.id); toggleGroup(group.id); }}
							>
								{#if isGroupExpanded(group.id)}
									<ChevronDown class="h-3.5 w-3.5 shrink-0 opacity-70" />
								{:else}
									<ChevronRight class="h-3.5 w-3.5 shrink-0 opacity-70" />
								{/if}
								<Folder class="h-4 w-4 shrink-0 opacity-70" />
								<div class="flex-1 min-w-0">
									<span class="text-sm font-medium truncate">{group.name}</span>
									{#if group.code}
										<span class="text-xs text-muted-foreground truncate ml-1">{group.code}</span>
									{/if}
								</div>
							</button>
							<div class="flex items-center gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity shrink-0">
								<Button
									variant="ghost"
									size="icon"
									class="h-6 w-6"
									onclick={(e) => openAddDir(group.id, e)}
									title={i18n.t('project.addDir')}
								>
									<FolderPlus class="h-3 w-3" />
								</Button>
								<Button
									variant="ghost"
									size="icon"
									class="h-6 w-6"
									onclick={(e) => { e.stopPropagation(); openEditProject(group); }}
									title={i18n.t('project.edit')}
								>
									<Pencil class="h-3 w-3" />
								</Button>
								<Button
									variant="ghost"
									size="icon"
									class="h-6 w-6 text-destructive hover:text-destructive"
									onclick={(e) => handleDeleteProject(group, e)}
									title={i18n.t('common.delete')}
								>
									<Trash2 class="h-3 w-3" />
								</Button>
							</div>
						</div>
					</div>
					<!-- 目录行 -->
					{#if isGroupExpanded(group.id)}
						{#each projectsStore.dirsOf(group.id) as dir (dir.id)}
							<button
								class={cn(
									'w-full text-left rounded-md pl-7 pr-2 py-1.5 group transition-colors',
									projectsStore.selectedId === dir.id
										? 'bg-sidebar-accent text-sidebar-accent-foreground'
										: 'hover:bg-sidebar-accent/50 text-sidebar-foreground'
								)}
								onclick={() => projectsStore.select(dir.id)}
								title={dir.path}
							>
								<div class="flex items-center gap-2">
									{#if dir.project_type === 'docs'}
										<FileText class="h-4 w-4 shrink-0 opacity-70" />
									{:else}
										<FolderGit2 class="h-4 w-4 shrink-0 opacity-70" />
									{/if}
									<div class="flex-1 min-w-0">
										<div class="text-sm truncate">{dir.name}</div>
										{#if dir.branch}
											<div class="text-xs text-muted-foreground truncate">{dir.branch}</div>
										{/if}
									</div>
									<div class="flex items-center gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity">
										<Button
											variant="ghost"
											size="icon"
											class="h-6 w-6"
											onclick={(e) => openEditDir(dir, e)}
											title={i18n.t('project.editDir')}
										>
											<Pencil class="h-3 w-3" />
										</Button>
										<Button
											variant="ghost"
											size="icon"
											class="h-6 w-6 text-destructive hover:text-destructive"
											onclick={(e) => handleDeleteDir(dir.id, e)}
											title={i18n.t('common.delete')}
										>
											<Trash2 class="h-3 w-3" />
										</Button>
									</div>
								</div>
							</button>
						{/each}
					{/if}
				{:else}
					<div class="text-xs text-muted-foreground text-center py-8 px-2">
						{i18n.t('project.emptyHint')}
					</div>
				{/each}
			</div>
		</ScrollArea>
	{:else}
		<div class="flex-1 flex flex-col items-center pt-2 gap-1">
			<Button variant="ghost" size="icon" class="h-8 w-8" onclick={openAddProject} title={i18n.t('project.add')}>
				<Plus class="h-4 w-4" />
			</Button>
			{#each projectsStore.groups as group (group.id)}
				<button
					class={cn(
						'w-8 h-8 flex items-center justify-center rounded-md transition-colors',
						projectsStore.selectedId === group.id
							? 'bg-sidebar-accent text-sidebar-accent-foreground'
							: 'hover:bg-sidebar-accent/50 text-sidebar-foreground'
					)}
					onclick={() => projectsStore.select(group.id)}
					title={group.name}
				>
					<Folder class="h-4 w-4" />
				</button>
			{/each}
		</div>
	{/if}
</div>

<!-- 项目 Dialog（仅名称 + 编号） -->
<Dialog bind:open={projectDialogOpen} onOpenChange={(v) => !v && resetProjectForm()}>
	<DialogContent class="sm:max-w-md">
		<DialogHeader>
			<DialogTitle>{editingProjectId ? i18n.t('project.edit') : i18n.t('project.add')}</DialogTitle>
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
		</div>
		<DialogFooter>
			<Button variant="outline" onclick={() => { projectDialogOpen = false; resetProjectForm(); }}>{i18n.t('common.cancel')}</Button>
			<Button onclick={handleSaveProject} disabled={!formName.trim()}>
				{i18n.t('common.save')}
			</Button>
		</DialogFooter>
	</DialogContent>
</Dialog>

<!-- 目录 Dialog -->
<Dialog bind:open={dirDialogOpen} onOpenChange={(v) => !v && resetDirForm()}>
	<DialogContent class="sm:max-w-md">
		<DialogHeader>
			<DialogTitle>{editingDirId ? i18n.t('project.editDir') : i18n.t('project.addDir')}</DialogTitle>
		</DialogHeader>
		<div class="grid gap-4 py-2">
			<div class="grid gap-2">
				<Label>{i18n.t('project.parent')} *</Label>
				<Select.Root type="single" bind:value={dirParentId}>
					<Select.Trigger class="w-full">
						{selectedParentLabel}
					</Select.Trigger>
					<Select.Content>
						{#each projectsStore.groups as group (group.id)}
							<Select.Item value={group.id} label={group.name}>
								{group.name}
							</Select.Item>
						{/each}
					</Select.Content>
				</Select.Root>
			</div>
			<div class="grid gap-2">
				<Label>{i18n.t('project.type')}</Label>
				<Select.Root type="single" bind:value={dirType}>
					<Select.Trigger class="w-full">
						{selectedTypeLabel}
					</Select.Trigger>
					<Select.Content>
						{#each typeItems as item (item.value)}
							<Select.Item value={item.value} label={i18n.t(item.labelKey)}>
								{i18n.t(item.labelKey)}
							</Select.Item>
						{/each}
					</Select.Content>
				</Select.Root>
			</div>
			<div class="grid gap-2">
				<Label for="dir-name">{i18n.t('project.dirName')} *</Label>
				<Input id="dir-name" bind:value={dirName} placeholder={i18n.t('project.dirName')} />
			</div>
			<div class="grid gap-2">
				<Label for="dir-path">{i18n.t('project.path')} *</Label>
				<div class="flex gap-2">
					<Input
						id="dir-path"
						bind:value={dirPath}
						placeholder={i18n.t('project.path')}
						class="flex-1"
						readonly
					/>
					<Button variant="outline" size="icon" onclick={pickFolder} title={i18n.t('common.selectFolder')}>
						<Folder class="h-4 w-4" />
					</Button>
				</div>
			</div>
			{#if dirType === 'code'}
				<div class="grid gap-2">
					<div class="flex items-center gap-2">
						<Label for="dir-git-user" class="flex-1">{i18n.t('project.gitUserName')}</Label>
						<span class="text-xs text-muted-foreground">{i18n.t('project.gitUserNameHint')}</span>
					</div>
					<Input
						id="dir-git-user"
						bind:value={dirGitUserName}
						placeholder={i18n.t('common.optional')}
					/>
				</div>
				<div class="grid gap-2">
					<div class="flex items-center gap-2">
						<Label class="flex-1">{i18n.t('project.branch')}</Label>
						<span class="text-xs text-muted-foreground">{i18n.t('project.branchHint')}</span>
					</div>
					<Popover.Root bind:open={branchPopoverOpen}>
						<Popover.Trigger
							class={cn(
								'border-input bg-background flex h-auto min-h-9 w-full items-center justify-between gap-2 rounded-md border px-3 py-2 text-sm shadow-xs',
								'focus-visible:border-ring focus-visible:ring-ring/50 focus-visible:ring-[3px] focus-visible:outline-none'
							)}
						>
							<span class={cn('text-left break-all', dirBranches.length === 0 && 'text-muted-foreground')}>
								{dirBranches.length > 0 ? dirBranches.join(', ') : i18n.t('project.branchPlaceholder')}
							</span>
							<ChevronsUpDown class="h-4 w-4 shrink-0 opacity-50" />
						</Popover.Trigger>
						<Popover.Content class="w-[var(--bits-popover-anchor-width)] p-0" align="start">
							<div class="p-2 border-b">
								<Input
									bind:value={branchFilter}
									placeholder={i18n.t('project.branchSearch')}
									class="h-8"
								/>
							</div>
							<div class="overflow-y-auto max-h-[min(16rem,var(--bits-popover-content-available-height))] p-1">
								{#each filteredBranches as branch (branch)}
									{@const selected = dirBranches.includes(branch)}
									<button
										class="w-full flex items-center gap-2 rounded-sm px-2 py-1.5 text-sm hover:bg-accent text-left"
										onclick={() => toggleBranch(branch)}
									>
										<Check class={cn('h-4 w-4 shrink-0', selected ? 'opacity-100' : 'opacity-0')} />
										<span class="truncate">{branch}</span>
									</button>
								{:else}
									<div class="px-2 py-1.5 text-xs text-muted-foreground">
										{i18n.t(branchLoadFailed ? 'project.branchLoadFailed' : (availableBranches.length === 0 ? 'project.branchEmpty' : 'project.branchNoMatch'))}
									</div>
								{/each}
							</div>
						</Popover.Content>
					</Popover.Root>
				</div>
			{/if}
		</div>
		<DialogFooter>
			<Button variant="outline" onclick={() => { dirDialogOpen = false; resetDirForm(); }}>{i18n.t('common.cancel')}</Button>
			<Button onclick={handleSaveDir} disabled={!dirName.trim() || !dirPath.trim() || !dirParentId}>
				{i18n.t('common.save')}
			</Button>
		</DialogFooter>
	</DialogContent>
</Dialog>
