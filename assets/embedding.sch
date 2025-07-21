v {xschem version=2.9.7 file_version=1.2}
C {TECHLIB/PCH} 620 -810 0 0 {name=x5 model=PCHLV w=4 l=0.09  m=1 embed=true}
[
v {xschem version=2.9.7 file_version=1.2}
G {}
K {type=pmos
format="@name @pinlist @model w=@w l=@l m=@m"
verilog_format="@verilog_gate #(@del ) @name ( @@d , @@s , @@g );"
template=" name=x1 verilog_gate=pmos del=50,50,50 model=PCH w=0.68 l=0.07 m=1 "
generic_type="model=string"
}
V {}
S {}
E {}
L 4 5 20 20 20 {}
L 4 20 20 20 30 {}
L 4 5 -20 20 -20 {}
L 4 20 -30 20 -20 {}
L 4 -20 0 -10 0 {}
L 4 5 -27.5 5 27.5 {11}
L 4 5 -5 10 0 {}
L 4 5 5 10 0 {}
L 4 10 0 20 0 {}
L 18 -2.5 -15 -2.5 15 {}
B 5 17.5 27.5 22.5 32.5 {name=d dir=inout}
B 5 -22.5 -2.5 -17.5 2.5 {name=g dir=in}
B 5 17.5 -32.5 22.5 -27.5 {name=s dir=inout}
B 5 17.5 -2.5 22.5 2.5 {name=b dir=in}
A 4 -6.25 0 3.75 270 360 {}
T {@w/@l*@m} 7.5 -17.5 0 0 0.2 0.2 {}
T {@name} 7.5 6.25 0 0 0.2 0.2 {999}
T {@model} 2.5 -27.5 0 1 0.2 0.2 {layer=8}
T {D} 25 17.5 0 0 0.15 0.15 {layer=13}
T {NF=@nf} -5 -15 0 1 0.15 0.15 {}
]
