<script lang="ts" generics="T">
    import * as Table from '$lib/components/ui/table';
    import * as Select from '$lib/components/ui/select';
    import {Button} from '$lib/components/ui/button';
    import {ChevronLeft, ChevronRight} from 'lucide-svelte';
    import type {Snippet} from 'svelte';
    import type {DataTableColumn} from './types';

    let {
        data,
        columns,
        total = 0,
        pageSize = $bindable(20),
        pageIndex = $bindable(1),
        selectedIds = $bindable<string[]>([]),
        rowKey = 'id',
        batchSelection = false,
        toolbar,
        header,
        body,
        cell,
        rowClass,
    }: {
        data: T[];
        columns?: DataTableColumn<T>[];
        total?: number;
        pageSize?: number;
        pageIndex?: number;
        selectedIds?: string[];
        rowKey?: string;
        batchSelection?: boolean;
        toolbar?: Snippet;
        header?: Snippet;
        body?: Snippet;
        cell?: Snippet<[{ column: DataTableColumn<T>; row: T; index: number }]>;
        rowClass?: (row: T) => string;
        onPageChange?: (page: number) => void;
        onPageSizeChange?: (size: number) => void;
    } = $props();

    const totalPages = $derived(Math.max(1, Math.ceil(total / pageSize)));

    const allPageSelected = $derived(batchSelection && data.length > 0 && data.every((row) => selectedIds.includes(String(row[rowKey as keyof T]))));

    const toggleSelectAll = () => {
        if (!batchSelection) return;
        const pageIds = data.map((row) => String(row[rowKey as keyof T]));
        if (allPageSelected) {
            selectedIds = selectedIds.filter((id) => !pageIds.includes(id));
        } else {
            const newIds = new Set([...selectedIds, ...pageIds]);
            selectedIds = Array.from(newIds);
        }
    };

    const toggleSelectRow = (id: string) => {
        if (!batchSelection) return;
        if (selectedIds.includes(id)) {
            selectedIds = selectedIds.filter((sid) => sid !== id);
        } else {
            selectedIds = [...selectedIds, id];
        }
    };

    const getCellStyle = (column: DataTableColumn<T>) => {
        const styles: string[] = [];
        if (column.width) {
            styles.push(`width: ${column.width}`);
        }
        if (column.align) {
            styles.push(`text-align: ${column.align}`);
        }
        return styles.join('; ');
    };
</script>

<div class="flex flex-col gap-2 h-full overflow-hidden">
    {#if toolbar}
        <div class="flex flex-wrap items-end gap-3 bg-white p-3 rounded-lg border border-gray-200">
            {@render toolbar()}
        </div>
    {/if}

    <Table.Root class="flex-1 overflow-auto bg-white rounded-lg border border-gray-200">
        {#if header}
            <Table.Header>
                {@render header()}
            </Table.Header>
        {:else if columns && columns.length > 0}
            <Table.Header class="sticky top-[1px] z-10">
                <Table.Row>
                    {#if batchSelection}
                        <Table.Head class="bg-gray-50 w-10">
                            <input
                                type="checkbox"
                                checked={allPageSelected}
                                onchange={toggleSelectAll}
                                class="h-4 w-4 rounded border-gray-300 text-primary focus:ring-primary"
                            />
                        </Table.Head>
                    {/if}
                    {#each columns as column}
                        <Table.Head
                                style={getCellStyle(column)}
                                class="bg-gray-50 {column.headerClass ?? ''}"
                        >
                            {column.title}
                        </Table.Head>
                    {/each}
                </Table.Row>
            </Table.Header>
        {/if}
        <Table.Body>
            {#if body}
                {@render body()}
            {:else if columns && columns.length > 0}
                {#each data as row, rowIndex}
                    <Table.Row class={rowClass ? rowClass(row) : ''}>
                        {#if batchSelection}
                            <Table.Cell class="w-10">
                                <input
                                    type="checkbox"
                                    checked={selectedIds.includes(String(row[rowKey as keyof T]))}
                                    onchange={() => toggleSelectRow(String(row[rowKey as keyof T]))}
                                    class="h-4 w-4 rounded border-gray-300 text-primary focus:ring-primary"
                                />
                            </Table.Cell>
                        {/if}
                        {#each columns as column}
                            <Table.Cell style={getCellStyle(column)} class={column.class}>
                                {#if column.slot && cell}
                                    {@render cell({column, row, index: rowIndex})}
                                {:else if column.formatter}
                                    {column.formatter(row[column.key as keyof T], row, rowIndex)}
                                {:else}
                                    {row[column.key as keyof T] ?? ''}
                                {/if}
                            </Table.Cell>
                        {/each}
                    </Table.Row>
                {:else}
                    <Table.Row>
                        <Table.Cell colspan={columns.length + (batchSelection ? 1 : 0)} class="text-center py-8 text-gray-400">
                            暂无数据
                        </Table.Cell>
                    </Table.Row>
                {/each}
            {/if}
        </Table.Body>
    </Table.Root>

    {#if total > 0}
        <div class="flex items-center justify-between bg-white p-3 rounded-lg border border-gray-200">
            <div class="text-sm text-gray-500">共 {total} 条</div>
            <div class="flex items-center gap-3">
                <Select.Root type="single" bind:value={pageSize} class="border border-gray-300 rounded-md px-2 py-1 text-sm bg-white">
                    <Select.Trigger>{pageSize}条/页</Select.Trigger>
                    <Select.Content>
                        <Select.Item value={10}>10条/页</Select.Item>
                        <Select.Item value={20}>20条/页</Select.Item>
                        <Select.Item value={50}>50条/页</Select.Item>
                        <Select.Item value={100}>100条/页</Select.Item>
                    </Select.Content>
                </Select.Root>
<!--                <Select-->
<!--                        class="border border-gray-300 rounded-md px-2 py-1 text-sm bg-white"-->
<!--                        bind:value={pageSize}-->
<!--                >-->
<!--                    <option value={10}>10条/页</option>-->
<!--                    <option value={20}>20条/页</option>-->
<!--                    <option value={50}>50条/页</option>-->
<!--                    <option value={100}>100条/页</option>-->
<!--                </Select>-->
                <div class="flex items-center gap-1">
                    <Button
                            variant="outline"
                            size="sm"
                            class="h-7 w-7 p-0"
                            disabled={pageIndex <= 1}
                            onclick={() => pageIndex = (Math.max(1, pageIndex - 1))}
                    >
                        <ChevronLeft class="h-4 w-4"/>
                    </Button>
                    <span class="text-sm px-2">{pageIndex} / {totalPages}</span>
                    <Button
                            variant="outline"
                            size="sm"
                            class="h-7 w-7 p-0"
                            disabled={pageIndex >= totalPages}
                            onclick={() => pageIndex = (Math.min(totalPages, pageIndex + 1))}
                    >
                        <ChevronRight class="h-4 w-4"/>
                    </Button>
                </div>
            </div>
        </div>
    {/if}
</div>
