CXX=g++
CXXFLAGS=-ggdb -Wall -std=c++14 -O0


all : spi

spi : spi.cpp
	${CXX} ${CXXFLAGS} $^ -o $@

clean :
	rm spi || true
