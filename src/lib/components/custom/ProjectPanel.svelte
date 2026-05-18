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
	import { Folder, Plus, Pencil, Trash2, ChevronLeft, ChevronRight, FolderGit2 } from '@lucide/svelte';
	import { cn } from '$lib/utils';

	let collapsed = $state(false);
	let dialogOpen = $state(false);
	let editingId = $state<string | null>(null);
	let formName = $state('');
	let formCode = $state('');
	let formPath = $state('');

	function resetForm() {
		formName = '';
		formCode = '';
		formPath = '';
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
			});
		} else {
			await projectsStore.add({
				name,
				code: formCode.trim() || undefined,
				path,
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
		'flex flex-col h-full border-r bg-sidebar transition-all duration-300',
		collapsed ? 'w-12' : 'w-64'
	)}
>
	<!-- Header -->
	<div class="flex items-center justify-between px-2 py-3 border-b shrink-0">
		{#if !collapsed}
			<span class="text-sm font-semibold text-sidebar-foreground truncate">项目</span>
		{/if}
		<div class="flex items-center gap-1">
			{#if !collapsed}
				<Button variant="ghost" size="icon" class="h-7 w-7" onclick={openAdd} title="添加项目">
					<Plus class="h-4 w-4" />
				</Button>
			{/if}
			<Button
				variant="ghost"
				size="icon"
				class="h-7 w-7"
				onclick={() => (collapsed = !collapsed)}
				title={collapsed ? '展开' : '收起'}
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
									title="编辑"
								>
									<Pencil class="h-3 w-3" />
								</Button>
								<Button
									variant="ghost"
									size="icon"
									class="h-6 w-6 text-destructive hover:text-destructive"
									onclick={(e) => handleDelete(project.id, e)}
									title="删除"
								>
									<Trash2 class="h-3 w-3" />
								</Button>
							</div>
						</div>
					</button>
				{:else}
					<div class="text-xs text-muted-foreground text-center py-8 px-2">
						暂无项目，点击上方 + 添加
					</div>
				{/each}
			</div>
		</ScrollArea>
	{:else}
		<div class="flex-1 flex flex-col items-center pt-2 gap-1">
			<Button variant="ghost" size="icon" class="h-8 w-8" onclick={openAdd} title="添加项目">
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
			<DialogTitle>{editingId ? '编辑项目' : '添加项目'}</DialogTitle>
		</DialogHeader>
		<div class="grid gap-4 py-2">
			<div class="grid gap-2">
				<Label for="project-name">项目名称 *</Label>
				<Input id="project-name" bind:value={formName} placeholder="输入项目名称" />
			</div>
			<div class="grid gap-2">
				<Label for="project-code">项目编号</Label>
				<Input id="project-code" bind:value={formCode} placeholder="可选" />
			</div>
			<div class="grid gap-2">
				<Label for="project-path">项目文件夹 *</Label>
				<div class="flex gap-2">
					<Input
						id="project-path"
						bind:value={formPath}
						placeholder="选择项目所在文件夹"
						class="flex-1"
						readonly
					/>
					<Button variant="outline" size="icon" onclick={pickFolder} title="选择文件夹">
						<Folder class="h-4 w-4" />
					</Button>
				</div>
			</div>
		</div>
		<DialogFooter>
			<Button variant="outline" onclick={() => { dialogOpen = false; resetForm(); }}>取消</Button>
			<Button onclick={handleSave} disabled={!formName.trim() || !formPath.trim()}>
				保存
			</Button>
		</DialogFooter>
	</DialogContent>
</Dialog>
