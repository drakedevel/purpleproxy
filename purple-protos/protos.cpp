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
  RewriterVisitor(Rewriter &R) : rewriter(R) {}

  bool VisitFunctionDecl(FunctionDecl *f) {
    std::stringstream ss;
    ss << rewriter.getRewrittenText(f->getSourceRange()) << std::endl;
    llvm::outs() << ss.str();
    return true;
  }

private:
  Rewriter &rewriter;
};

class RewriterConsumer : public ASTConsumer {
public:
  RewriterConsumer(Rewriter &R) : visitor(R) {}

  virtual void HandleTranslationUnit(ASTContext &ctx) {
    Decl *tu = ctx.getTranslationUnitDecl();
    visitor.TraverseDecl(tu);
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
  CommonOptionsParser op(argc, argv, ProtosCategory);
  ClangTool Tool(op.getCompilations(), op.getSourcePathList());

  return Tool.run(newFrontendActionFactory<RewriterAction>().get());
}
