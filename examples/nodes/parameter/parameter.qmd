A boolean parameter &[par_bool_1]{bool}, with a default &[par_bool_2]{bool def=true}, with a default and value &[par_bool_3]{bool def=false}

An integer parameter &[par_int_1]{int}, with a default &[par_int_2]{int def=123}, with a default and value &[par_int_3]{int def=123}, with min, max and mult options &[par_int_4]{int def=123 min=-100 max=100 mult=2}

A number parameter &[par_num_1]{num def=1.23 min=-10.1 emin=-10.1 max=10.1 emax=10.1 mult=2.2}

A string parameter &[par_str_1]{str def="hello" minlen=0 maxlen=10 pattern="[a-z]+"} with quotes in properties &[par_str_2]{str def='a"b' pattern='a"b'}

A date parameter &[par_date]{date def="2022-02-22" min="2022-02-20" max="2022-02-24"}

A time parameter &[par_time]{time def="22:22" min="22:20" max="22:24"}

A date-time parameter &[par_date_time]{datetime def="2022-02-22T22:22" min="2022-02-22T22:20" max="2022-02-22T22:24"}

An enum parameter &[par_enum]{enum def="green" vals=["blue","red","green"]}
