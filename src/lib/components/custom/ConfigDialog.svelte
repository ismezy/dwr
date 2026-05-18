<script lang="ts">
	import { open } from '@tauri-apps/plugin-dialog';
	import { Button } from '$lib/components/ui/button/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import { Label } from '$lib/components/ui/label/index.js';
	import {
		Dialog,
		DialogContent,
		DialogHeader,
		DialogTitle,
		DialogFooter,
	} from '$lib/components/ui/dialog/index.js';
	import { configStore } from '$lib/stores/config.svelte';
	import { Folder, Settings } from '@lucide/svelte';

	let openState = $state(false);
	let workDir = $state('');
	let gitUserName = $state('');

	async function show() {
		await configStore.refresh();
		workDir = configStore.configs.work_dir ?? '';
		gitUserName = configStore.configs.git_user_name ?? '';
		openState = true;
	}

	async function pickWorkDir() {
		const selected = await open({
			directory: true,
			multiple: false,
		});
		if (selected && typeof selected === 'string') {
			workDir = selected;
		}
	}

	async function handleSave() {
		await configStore.save({
			work_dir: workDir.trim() || undefined,
			git_user_name: gitUserName.trim() || undefined,
		});
		openState = false;
	}
</script>

<Button variant="ghost" size="icon" class="h-8 w-8" onclick={show} title="全局配置">
	<Settings class="h-4 w-4" />
</Button>

<Dialog bind:open={openState}>
	<DialogContent class="sm:max-w-md">
		<DialogHeader>
			<DialogTitle>全局配置</DialogTitle>
		</DialogHeader>
		<div class="grid gap-5 py-2">
			<div class="grid gap-2">
				<Label for="work-dir">工作目录（保存日报文件）</Label>
				<div class="flex gap-2">
					<Input
						id="work-dir"
						bind:value={workDir}
						placeholder="选择日报保存目录"
						class="flex-1"
						readonly
					/>
					<Button variant="outline" size="icon" onclick={pickWorkDir} title="选择文件夹">
						<Folder class="h-4 w-4" />
					</Button>
				</div>
			</div>
			<div class="grid gap-2">
				<Label for="git-user-name">提交记录用户名称</Label>
				<Input
					id="git-user-name"
					bind:value={gitUserName}
					placeholder="用于过滤 Git 提交记录"
				/>
			</div>
		</div>
		<DialogFooter>
			<Button variant="outline" onclick={() => (openState = false)}>取消</Button>
			<Button onclick={handleSave}>保存</Button>
		</DialogFooter>
	</DialogContent>
</Dialog>
