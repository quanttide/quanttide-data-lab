keyword: {
	category:    =~"^C[0-9]{2}$"
	keyword:     string & !=""
	variant_type: "direct" | "homophone" | "pinyin" | "split"
	parent_id?:  string
	transform_rule: string & !="" // 变换规则标注，规则类变体必填，direct 填 identity
	source:      string & !=""
	source_date: =~"^[0-9]{4}-[0-9]{2}-[0-9]{2}$"
	version:     string & !=""
	run_id:      string & !=""
}
