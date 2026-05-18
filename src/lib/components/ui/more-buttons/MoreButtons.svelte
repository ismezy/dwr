<script lang="ts">
	import { onMount } from 'svelte';
	import { MoreVertical } from 'lucide-svelte';
	import { Button } from '$lib/components/ui/button';

	interface Props {
		showCount?: number;
		title?: string;
		children?: import('svelte').Snippet;
	}

	let { showCount = 3, title = '', children }: Props = $props();

	let containerRef: HTMLDivElement;
	let dropdownRef: HTMLDivElement;
	let moreBtnRef: HTMLButtonElement | null = $state(null);
	let hasMore = $state(false);
	let isOpen = $state(false);
	let isRecalculating = false;

	function getContentChildren(): HTMLElement[] {
		if (!containerRef) return [];
		return Array.from(containerRef.children).filter(
			(el) =>
				el !== moreBtnRef && !(el as HTMLElement).classList.contains('more-buttons-trigger')
		) as HTMLElement[];
	}

	function recalc() {
		if (!containerRef || !dropdownRef || isRecalculating) return;
		isRecalculating = true;

		try {
			// Return previously moved items back to container
			const dropdownChildren = Array.from(dropdownRef.children) as HTMLElement[];
			dropdownChildren.forEach((child) => {
				child.style.display = '';
				containerRef.appendChild(child);
			});

			const allChildren = getContentChildren();
			const count = allChildren.length;
			hasMore = count > showCount;

			if (hasMore) {
				const overflow = allChildren.slice(showCount);
				overflow.forEach((child) => {
					child.style.display = '';
					dropdownRef.appendChild(child);
				});
			}
		} finally {
			isRecalculating = false;
		}
	}

	onMount(() => {
		if (!containerRef) return;

		// Initial calc with a small delay to ensure children are rendered
		const timeout = setTimeout(recalc, 0);

		// Watch for children changes (e.g. dynamic snippet re-rendering)
		const observer = new MutationObserver(() => {
			if (!isRecalculating) {
				requestAnimationFrame(recalc);
			}
		});
		observer.observe(containerRef, { childList: true, subtree: false });

		return () => {
			clearTimeout(timeout);
			observer.disconnect();
		};
	});

	function toggleDropdown() {
		isOpen = !isOpen;
		if (isOpen) {
			positionDropdown();
		}
	}

	function positionDropdown() {
		if (!moreBtnRef || !dropdownRef) return;
		const rect = moreBtnRef.getBoundingClientRect();
		const dropdownWidth = dropdownRef.offsetWidth || 120;
		const dropdownHeight = dropdownRef.offsetHeight || 80;
		let left = rect.left;
		let top = rect.bottom + 4;
		if (left + dropdownWidth > window.innerWidth) {
			left = rect.right - dropdownWidth;
		}
		if (top + dropdownHeight > window.innerHeight) {
			top = rect.top - dropdownHeight - 4;
		}
		dropdownRef.style.top = `${top}px`;
		dropdownRef.style.left = `${left}px`;
	}

	function handleWindowClick(e: MouseEvent) {
		if (!isOpen) return;
		const target = e.target as Node;
		if (
			dropdownRef?.contains(target) ||
			moreBtnRef?.contains(target as Node) ||
			moreBtnRef === target
		) {
			return;
		}
		isOpen = false;
	}

	function handleDropdownClick(e: MouseEvent) {
		const target = e.target as HTMLElement;
		if (target.closest('button, a')) {
			isOpen = false;
		}
	}
</script>

<svelte:window onclick={handleWindowClick} />

<div class="more-buttons-container" bind:this={containerRef}>
	{@render children?.()}
	{#if hasMore}
		<Button
			bind:ref={moreBtnRef}
			variant="ghost"
			size="sm"
			class="h-6 w-6 p-0 more-buttons-trigger"
			onclick={toggleDropdown}
		>
			{#if title}
				<span class="text-xs text-primary">{title}</span>
			{:else}
				<MoreVertical class="h-3.5 w-3.5" />
			{/if}
		</Button>
	{/if}
</div>

<div
	bind:this={dropdownRef}
	class="more-buttons-dropdown"
	class:open={isOpen}
	onclick={handleDropdownClick}
	onkeydown={(e) => {
		if (e.key === 'Escape') isOpen = false;
	}}
	role="menu"
	tabindex="-1"
>
	<!-- overflow buttons will be moved here via DOM -->
</div>

<style>
	.more-buttons-container {
		display: inline-flex;
		align-items: center;
		gap: 0.25rem;
		position: relative;
	}

	.more-buttons-dropdown {
		display: none;
		position: fixed;
		background: var(--background);
		border: 1px solid var(--border);
		border-radius: var(--radius-md);
		box-shadow: 0 10px 15px -3px rgba(0, 0, 0, 0.1), 0 4px 6px -4px rgba(0, 0, 0, 0.1);
		z-index: 9999;
		min-width: 120px;
		padding: 0.25rem;
		flex-direction: column;
		gap: 0.125rem;
	}

	.more-buttons-dropdown.open {
		display: flex;
	}

	.more-buttons-dropdown :global(button),
	.more-buttons-dropdown :global(a) {
		justify-content: flex-start !important;
		width: 100%;
	}
</style>
