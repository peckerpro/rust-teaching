# 项目目录结构

```
rust-teaching/
├── Cargo.toml                    # workspace根
├── Cargo.lock
├── Dockerfile
├── docker-compose.yml
├── .github/
│   └── workflows/
│       ├── ci.yml                # 编译+测试+Lint
│       └── deploy.yml            # 构建发布
│
├── crates/                       # 编译器核心（Rust workspace）
│   ├── common/                   # 共享基础类型
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── span.rs           # 源码位置(Span)
│   │       ├── token.rs          # Token定义
│   │       ├── symbol.rs         # 内部字符串池(Interner)
│   │       └── diagnostic.rs     # 错误/警告信息结构
│   │
│   ├── lexer/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── lexer.rs          # 词法分析器主逻辑
│   │       ├── cursor.rs         # 字符读取光标
│   │       └── literal.rs        # 字面量解析(数字/字符串)
│   │
│   ├── parser/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── parser.rs         # 语法分析器入口
│   │       ├── expr.rs           # 表达式解析(Pratt)
│   │       ├── stmt.rs           # 语句解析
│   │       ├── item.rs           # 声明解析(fn/struct/enum/impl)
│   │       ├── pattern.rs        # 模式解析
│   │       ├── ty.rs             # 类型标注解析
│   │       └── path.rs           # 路径解析
│   │
│   ├── ast/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── node.rs           # AST节点枚举(带Span)
│   │       ├── expr.rs           # 表达式节点
│   │       ├── stmt.rs           # 语句节点
│   │       ├── item.rs           # 声明节点
│   │       ├── pattern.rs        # 模式节点
│   │       ├── ty.rs             # 类型节点
│   │       ├── visit.rs          # 访问者trait(Visitor/MutVisitor)
│   │       └── display.rs        # AST Pretty Print
│   │
│   ├── semantic/                 # 语义分析
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── resolve.rs        # 名称解析 + 作用域
│   │       ├── scope.rs          # 作用域栈/符号表
│   │       ├── typeck.rs         # 类型检查
│   │       ├── borrowck.rs       # 借用检查
│   │       ├── ty.rs             # 语义类型(TyKind)
│   │       └── infer.rs          # 类型推断(Unification)
│   │
│   ├── codegen/                  # LLVM代码生成
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── context.rs        # LLVM上下文管理
│   │       ├── codegen.rs        # AST → LLVM IR
│   │       ├── expr.rs           # 表达式代码生成
│   │       ├── stmt.rs           # 语句代码生成
│   │       ├── ty.rs             # 类型→LLVM类型映射
│   │       └── intrinsic.rs      # 内置函数/Intrinsic
│   │
│   ├── driver/                   # 编译器驱动
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── session.rs        # 编译会话(配置+诊断)
│   │       └── pipeline.rs       # 编译流水线编排
│   │
│   └── cli/                      # 命令行工具
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs
│           └── args.rs           # clap参数定义
│
├── services/                     # 微服务（独立crate）
│   ├── compiler-service/
│   │   ├── Cargo.toml
│   │   ├── Dockerfile
│   │   └── src/
│   │       ├── main.rs
│   │       ├── routes/
│   │       │   ├── mod.rs
│   │       │   ├── compile.rs    # POST /compile
│   │       │   └── ws.rs         # WebSocket
│   │       ├── sandbox/
│   │       │   └── mod.rs        # Docker沙箱执行
│   │       └── state.rs          # 应用状态
│   │
│   ├── teaching-service/
│   │   ├── Cargo.toml
│   │   ├── Dockerfile
│   │   └── src/
│   │       ├── main.rs
│   │       ├── routes/
│   │       ├── models/
│   │       └── db.rs
│   │
│   ├── user-service/
│   │   ├── Cargo.toml
│   │   ├── Dockerfile
│   │   └── src/
│   │       ├── main.rs
│   │       ├── routes/
│   │       ├── models/
│   │       └── auth.rs
│   │
│   └── api-gateway/
│       ├── nginx.conf
│       └── Dockerfile
│
├── frontend/                     # 教学平台前端
│   ├── package.json
│   ├── vite.config.ts
│   ├── tsconfig.json
│   ├── index.html
│   └── src/
│       ├── main.ts
│       ├── App.vue
│       ├── router/
│       ├── views/
│       │   ├── EditorView.vue       # 代码编辑器
│       │   ├── CompileView.vue      # 编译过程展示
│       │   ├── ASTView.vue          # AST可视化
│       │   ├── IRView.vue           # LLVM IR展示
│       │   ├── CourseView.vue       # 课程列表
│       │   ├── ExerciseView.vue     # 练习题
│       │   └── ProgressView.vue     # 学习进度
│       ├── components/
│       │   ├── MonacoEditor.vue     # 代码编辑器组件
│       │   ├── TokenStream.vue      # Token流展示
│       │   ├── ASTTree.vue          # AST树可视化
│       │   ├── SymbolTable.vue      # 符号表展示
│       │   ├── CFGView.vue          # 控制流图
│       │   ├── IRCode.vue           # IR代码块
│       │   └── CompileProgress.vue  # 编译进度
│       └── api/
│           ├── index.ts
│           ├── compiler.ts
│           ├── teaching.ts
│           └── user.ts
│
├── tests/                        # 集成测试
│   ├── compile_tests/            # 编译正确性测试用例
│   │   ├── hello.rs
│   │   ├── fibonacci.rs
│   │   ├── structs.rs
│   │   ├── enums.rs
│   │   ├── generics.rs
│   │   └── traits.rs
│   └── error_tests/              # 编译错误测试
│       ├── type_error.rs
│       └── borrow_error.rs
│
├── docs/                         # 文档
│   ├── 01-requirements.md        # 需求清单（本文档姊妹篇）
│   ├── 02-tech-stack.md          # 技术栈方案
│   ├── 03-project-structure.md   # 项目结构（本文件）
│   ├── 04-grammar.md             # 支持的语言语法(EBNF)
│   ├── 05-compiler-design.md     # 编译器设计文档
│   └── 06-course-outline.md      # 课程大纲
│
├── scripts/                      # 工具脚本
│   ├── setup-dev.sh              # 开发环境一键配置
│   └── test-all.sh               # 全量测试脚本
│
└── .vscode/                      # 编辑器配置(可选)
    └── settings.json
```
