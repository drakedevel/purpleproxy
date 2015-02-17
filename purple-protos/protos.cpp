#include <sstream>
#include <string>

#include "clang/AST/AST.h"
#include "clang/AST/ASTConsumer.h"
#include "clang/AST/RecursiveASTVisitor.h"
#include "clang/Frontend/ASTConsumers.h"
#include "clang/Frontend/FrontendActions.h"
#include "clang/Frontend/CompilerInstance.h"
#include "clang/Tooling/CommonOptionsParser.h"
#include "clang/Tooling/Tooling.h"
#include "clang/Rewrite/Core/Rewriter.h"
#include "llvm/Support/raw_ostream.h"

static llvm::cl::OptionCategory ProtosCategory("Protos");

using namespace clang;
using namespace clang::driver;
using namespace clang::tooling;

class RewriterVisitor : public RecursiveASTVisitor<RewriterVisitor> {
public:
  RewriterVisitor(Rewriter &R) : first(true), rewriter(R) {}

  bool VisitFunctionDecl(FunctionDecl *f) {
    std::stringstream ss;
    if (rewriter.getSourceMgr().getFilename(f->getLocation()).find("/libpurple/") == std::string::npos)
        return true;

    if (first)
      first = false;
    else
      ss << ", ";
    ss << "{\"name\":\"" << f->getNameInfo().getAsString() << "\",";
    ss << "\"variadic\":" << (f->isVariadic() ? "true" : "false") << ",";
#ifdef LLVM_34
    ss << "\"return\":" << jsonQualType(f->getResultType()) << ",";
#else
    ss << "\"return\":" << jsonQualType(f->getReturnType()) << ",";
#endif
    ss << "\"params\":[";
    bool first = true;
    for (auto i = f->param_begin(); i != f->param_end(); ++i) {
      if (first)
        first = false;
      else
        ss << ", ";
      ss << "{\"name\": \"" << (*i)->getName().str() << "\", \"type\":" << jsonQualType((*i)->getOriginalType()) << "}";
    }
    ss << "]}";
    llvm::outs() << ss.str();
    return true;
  }

private:
  bool first;
  std::string jsonQualType(QualType type) {
    std::stringstream ss;
    ss << "\"" << type.getAsString() << "\"";
    return ss.str();
  }

  Rewriter &rewriter;
};

class RewriterConsumer : public ASTConsumer {
public:
  RewriterConsumer(Rewriter &R) : visitor(R) {}

  virtual void HandleTranslationUnit(ASTContext &ctx) {
    llvm::outs() << "{\"functions\":[";
    Decl *tu = ctx.getTranslationUnitDecl();
    visitor.TraverseDecl(tu);
    llvm::outs() << "]}\n";
  }

private:
  RewriterVisitor visitor;
};

class RewriterAction : public ASTFrontendAction {
public:
  RewriterAction() {}

  virtual ASTConsumer *CreateASTConsumer(CompilerInstance &CI, StringRef file) {
    rewriter.setSourceMgr(CI.getSourceManager(), CI.getLangOpts());
    return new RewriterConsumer(rewriter);
  }

private:
  Rewriter rewriter;
};

int main(int argc, const char **argv) {
#ifdef LLVM_34
  CommonOptionsParser op(argc, argv);
  ClangTool Tool(op.getCompilations(), op.getSourcePathList());

  return Tool.run(newFrontendActionFactory<RewriterAction>());
#else
  CommonOptionsParser op(argc, argv, ProtosCategory);
  ClangTool Tool(op.getCompilations(), op.getSourcePathList());

  return Tool.run(newFrontendActionFactory<RewriterAction>().get());
#endif
}
