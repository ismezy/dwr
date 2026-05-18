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
	import * as Select from '$lib/components/ui/select/index.js';
	import { configStore, type Theme } from '$lib/stores/config.svelte';
	import { i18n, type Locale } from '$lib/i18n';
	import { Folder, Settings } from '@lucide/svelte';

	let openState = $state(false);
	let workDir = $state('');
	let gitUserName = $state('');
	let lang = $state<Locale>('zh');
	let theme = $state<Theme>('system');

	async function show() {
		await configStore.refresh();
		workDir = configStore.configs.work_dir ?? '';
		gitUserName = configStore.configs.git_user_name ?? '';
		lang = configStore.configs.lang ?? 'zh';
		theme = configStore.configs.theme ?? 'system';
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
			lang,
			theme,
		});
		openState = false;
	}

	const langItems = $derived([
		{ value: 'zh' as Locale, label: i18n.t('config.lang.zh') },
		{ value: 'en' as Locale, label: i18n.t('config.lang.en') },
	]);

	const themeItems = $derived([
		{ value: 'light' as Theme, label: i18n.t('config.theme.light') },
		{ value: 'dark' as Theme, label: i18n.t('config.theme.dark') },
		{ value: 'system' as Theme, label: i18n.t('config.theme.system') },
	]);

	const selectedLangLabel = $derived(langItems.find((i) => i.value === lang)?.label ?? '');
	const selectedThemeLabel = $derived(themeItems.find((i) => i.value === theme)?.label ?? '');
</script>

<Button variant="ghost" size="icon" class="h-8 w-8" onclick={show} title={i18n.t('config.title')}>
	<Settings class="h-4 w-4" />
</Button>

<Dialog bind:open={openState}>
	<DialogContent class="sm:max-w-md">
		<DialogHeader>
			<DialogTitle>{i18n.t('config.title')}</DialogTitle>
		</DialogHeader>
		<div class="grid gap-5 py-2">
			<div class="grid gap-2">
				<Label for="work-dir">{i18n.t('config.workDir')}</Label>
				<div class="flex gap-2">
					<Input
						id="work-dir"
						bind:value={workDir}
						placeholder={i18n.t('config.workDir')}
						class="flex-1"
						readonly
					/>
					<Button variant="outline" size="icon" onclick={pickWorkDir} title="选择文件夹">
						<Folder class="h-4 w-4" />
					</Button>
				</div>
			</div>
			<div class="grid gap-2">
				<Label for="git-user-name">{i18n.t('config.gitUserName')}</Label>
				<Input
					id="git-user-name"
					bind:value={gitUserName}
					placeholder={i18n.t('config.gitUserName')}
				/>
			</div>
			<div class="grid gap-2">
				<Label>{i18n.t('config.language')}</Label>
				<Select.Root type="single" bind:value={lang}>
					<Select.Trigger class="w-full">
						{selectedLangLabel}
					</Select.Trigger>
					<Select.Content>
						{#each langItems as item (item.value)}
							<Select.Item value={item.value} label={item.label}>
								{item.label}
							</Select.Item>
						{/each}
					</Select.Content>
				</Select.Root>
			</div>
			<div class="grid gap-2">
				<Label>{i18n.t('config.theme')}</Label>
				<Select.Root type="single" bind:value={theme}>
					<Select.Trigger class="w-full">
						{selectedThemeLabel}
					</Select.Trigger>
					<Select.Content>
						{#each themeItems as item (item.value)}
							<Select.Item value={item.value} label={item.label}>
								{item.label}
							</Select.Item>
						{/each}
					</Select.Content>
				</Select.Root>
			</div>
		</div>
		<DialogFooter>
			<Button variant="outline" onclick={() => (openState = false)}>{i18n.t('common.cancel')}</Button>
			<Button onclick={handleSave}>{i18n.t('common.save')}</Button>
		</DialogFooter>
	</DialogContent>
</Dialog>
