Examples of the parameter type can be inferred from options when using Markdown.

No options and no type, so no validator: &[par1].

Options only found on string parameters: &[par1]{minlen=3}, &[par2]{maxlen=5}, &[par3]{pattern="[a-z]{3,5}"}.

Option `vals` is only on "enum" parameters: &[par1]{values=[1,2,3]}.

Options specified as bool: &[par1]{def=true}.

Options specified as integer: &[par1]{def=1}, &[par2]{min=2}, &[par3]{max=3}.

Options specified as number: &[par1]{def=1.1}, &[par2]{emin=2.2}, &[par3]{emax=3.3}, &[par4]{step=0.1}.

Options specified as date: &[par1]{min=2022-02-22}.

Options specified as time: &[par1]{max=01:02:03}.

Options specified as datetime: &[par1]{min=2022-02-22T01:02:03}.
