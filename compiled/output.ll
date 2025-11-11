; ModuleID = 'adan_module'
source_filename = "adan_module"

@str = global [36 x i8] c"ADAN compiled and ran successfully.\00"
@fmt_str = global [5 x i8] c"%s\0A\00\00"

define double @main() {
entry:
  %call_printf = call i32 (ptr, ...) @printf(ptr @fmt_str, ptr @str)
  ret double 0.000000e+00
}

declare i32 @printf(ptr, ...)
