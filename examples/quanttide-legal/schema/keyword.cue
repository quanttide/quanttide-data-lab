keyword: {
	category:    =~"^C[0-9]{2}$"
	keyword:     string & !=""
	variant_type: "direct" | "homophone" | "pinyin" | "split"
	parent_id?:  string
	source:      string & !=""
	source_date: =~"^[0-9]{4}-[0-9]{2}-[0-9]{2}$"
	version:     string & !=""
	run_id:      string & !=""
}
