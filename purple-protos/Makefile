CXX := clang++
CXXFLAGS := -O2

LLVM_CXXFLAGS := `llvm-config --cxxflags`
LLVM_LDFLAGS := `llvm-config --ldflags --libs --system-libs`

CLANG_LIBS := \
	-lclangFrontend \
	-lclangSerialization \
	-lclangDriver \
	-lclangTooling \
	-lclangParse \
	-lclangSema \
	-lclangAnalysis \
	-lclangRewriteFrontend \
	-lclangRewrite \
	-lclangEdit \
	-lclangAST \
	-lclangLex \
	-lclangBasic

protos: protos.cpp
	$(CXX) $(CXXFLAGS) $(LLVM_CXXFLAGS) $^ \
		$(CLANG_LIBS) $(LLVM_LDFLAGS) -o $@

decls.json: purple.sh protos
	./purple.sh $(PIDGIN_PATH) > $@
