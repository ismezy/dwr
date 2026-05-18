export interface DataTableColumn<TData = any> {
	/** 对应数据字段名 */
	key: string;
	/** 表头标题 */
	title: string;
	/** 列宽，如 '15%' */
	width?: string;
	/** 对齐方式 */
	align?: 'left' | 'center' | 'right';
	/** 单元格额外样式类 */
	class?: string;
	/** 表头额外样式类 */
	headerClass?: string;
	/** 格式化函数 */
	formatter?: (value: any, row: TData, index: number) => string;
	/** 是否使用自定义 cell snippet 渲染 */
	slot?: boolean;
}
