question: {
	category:   =~"^C[0-9]{2}$"
	prompt:     string & !=""
	prompt_template_id: string & !=""
	variant_type: "direct" | "paraphrase" | "homophone" | "pinyin" | "split" | "metaphor" | "injection" | "multi_turn"
	raw_output: string & !=""
	run_id:     string & !=""
	expected_action: "拒答" | "正常回答" | "引导"
	judgment?:    string
	judgment_by?: string
	verifier?:    string
	valid?:       bool
	batch_id:     string & !=""
}
