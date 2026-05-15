# Rust 教学编译器 - 支持语法 (EBNF)

## 程序结构
```
program        ::= item*
```

## 声明 (Items)
```
item           ::= fn_item
                 | struct_item
                 | enum_item
                 | impl_item
                 | trait_item
                 | mod_item
                 | use_item
                 | const_item
                 | static_item
                 | macro_rules_item

fn_item        ::= 'fn' IDENT generic_params? '(' fn_params? ')' return_type? block_expr

struct_item    ::= 'struct' IDENT generic_params? ( struct_fields | tuple_fields | ';' )

enum_item      ::= 'enum' IDENT generic_params? '{' enum_variants? '}'

impl_item      ::= 'impl' generic_params? (type | trait_type) '{' impl_member* '}'

trait_item     ::= 'trait' IDENT generic_params? '{' trait_member* '}'

mod_item       ::= 'mod' IDENT ';'           (* 外部模块 *)
                 | 'mod' IDENT '{' item* '}'  (* 内联模块 *)

use_item       ::= 'use' use_path ';'

const_item     ::= 'const' IDENT ':' type '=' expr ';'

static_item    ::= 'static' ('mut')? IDENT ':' type '=' expr ';'
```

## 表达式
```
expr           ::= assign_expr

assign_expr    ::= range_expr ('=' range_expr)?

range_expr     ::= or_expr ('..' or_expr)? ('..=' or_expr)?

or_expr        ::= and_expr ('||' and_expr)*

and_expr       ::= cmp_expr ('&&' cmp_expr)*

cmp_expr       ::= bit_or_expr (('==' | '!=' | '<' | '>' | '<=' | '>=') bit_or_expr)?

bit_or_expr    ::= bit_xor_expr ('|' bit_xor_expr)*

bit_xor_expr   ::= bit_and_expr ('^' bit_and_expr)*

bit_and_expr   ::= shift_expr ('&' shift_expr)*

shift_expr     ::= add_expr (('<<' | '>>') add_expr)*

add_expr       ::= mul_expr (('+' | '-') mul_expr)*

mul_expr       ::= unary_expr (('*' | '/' | '%') unary_expr)*

unary_expr     ::= ('-' | '!' | '*' | '&' | '&mut') unary_expr
                 | call_expr

call_expr      ::= field_expr ('(' args? ')')?
                 | field_expr '[' expr ']' field_expr?

field_expr     ::= primary_expr ('.' IDENT)*
                 | primary_expr '::' IDENT

primary_expr   ::= literal
                 | IDENT
                 | '(' expr ')'
                 | '{' block_stmts? '}'
                 | 'if' expr block_expr ('else' (block_expr | if_expr))?
                 | 'loop' block_expr
                 | 'while' expr block_expr
                 | 'for' pattern 'in' expr block_expr
                 | 'match' expr '{' match_arms '}'
                 | 'return' expr?
                 | 'break' expr?
                 | 'continue'
                 | closure_expr

closure_expr   ::= '|' params? '|' expr
                 | 'move' '|' params? '|' expr
```

## 语句
```
block_expr     ::= '{' stmt* expr? '}'

block_stmts    ::= stmt+ expr?

stmt           ::= let_stmt
                 | expr_stmt
                 | item_stmt
                 | ';'

let_stmt       ::= 'let' pattern (':' type)? ('=' expr)? ';'

expr_stmt      ::= expr ';'
```

## 模式
```
pattern        ::= ident_pattern
                 | literal_pattern
                 | wildcard_pattern
                 | ref_pattern
                 | struct_pattern
                 | tuple_pattern
                 | enum_pattern
                 | or_pattern

ident_pattern  ::= ('mut' | 'ref' 'mut'?)? IDENT

literal_pat    ::= INTEGER | FLOAT | BOOL | CHAR | STRING

wildcard_pat   ::= '_'

ref_pattern    ::= '&' pattern
                 | '&mut' pattern

struct_pat     ::= path '{' (field_pat ',')* '..'? '}'

tuple_pat      ::= '(' (pattern ',')* '..'? ')'

enum_pat       ::= path '(' (pattern ',')* ')'

or_pattern     ::= pattern '|' pattern
```

## 类型
```
type           ::= path_type
                 | ref_type
                 | tuple_type
                 | array_type
                 | slice_type
                 | fn_type
                 | unit_type

path_type      ::= path (generic_args)?

ref_type       ::= '&' lifetime? ('mut')? type

tuple_type     ::= '(' (type ',')+ ')'

array_type     ::= '[' type ';' expr ']'

slice_type     ::= '[' type ']'

fn_type        ::= 'fn' '(' fn_params? ')' return_type?

unit_type      ::= '(' ')'
```

## 字面量
```
literal        ::= INTEGER | FLOAT | BOOL | CHAR | STRING | BYTE | BYTE_STRING

INTEGER        ::= DECIMAL | HEX | OCTAL | BINARY
DECIMAL        ::= [0-9]+ ('_' [0-9]+)* (INT_SUFFIX)?
HEX            ::= '0x' [0-9a-fA-F]+ ('_' [0-9a-fA-F]+)*
OCTAL          ::= '0o' [0-7]+ ('_' [0-7]+)*
BINARY         ::= '0b' [01]+ ('_' [01]+)*
FLOAT          ::= [0-9]+ '.' [0-9]+ (EXPONENT)? (FLOAT_SUFFIX)?
BOOL           ::= 'true' | 'false'
CHAR           ::= '\'' char_content '\''
STRING         ::= '"' string_content* '"'
BYTE           ::= 'b\'' char_content '\''
BYTE_STRING    ::= 'b"' string_content* '"'
```

## 旁路结构
```
path           ::= 'self' | 'super' | 'crate' | IDENT
                 | path '::' IDENT

use_path       ::= path '::'? '{' use_item_inner* '}'
                 | path ('as' IDENT)?

generic_params ::= '<' (lifetime ',')* (type_param ',')* lifetime? type_param? '>'

generic_args   ::= '<' (type ',')* type '>'

fn_params      ::= param (',' param)*

param          ::= pattern ':' type

args           ::= expr (',' expr)*

return_type    ::= '->' type

match_arms     ::= match_arm (',' match_arm)* ','?

match_arm      ::= pattern ('if' expr)? '=>' expr
```

## 关键字列表
```
ABSTRACT | AS | ASYNC | AWAIT | BECOME | BOX | BREAK | CONST | CONTINUE
CRATE | DO | DYN | ELSE | ENUM | EXTERN | FALSE | FINAL | FN | FOR | GEN
IF | IMPL | IN | LET | LOOP | MACRO | MATCH | MOD | MOVE | MUT | OVERRIDE
PRIV | PUB | RAW | REF | RETURN | SELF | STATIC | STRUCT | SUPER | TRAIT
TRUE | TRY | TYPE | UNION | UNSAFE | UNSIZED | USE | VIRTUAL | WHERE | WHILE | YIELD
```

> **教学编译器简化说明**：
> - 不支持：`unsafe`、`extern`、原始标识符(r#)、属性(保留解析但不做语义)
> - 简化：泛型默认参数字、where子句、关联类型、GATs
> - 不支持：async/await、const泛型、impl Trait语法、let-else
> - 模式匹配：不支持切片模式、范围模式(@语法仅做解析)
