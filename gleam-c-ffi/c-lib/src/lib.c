#include <stdio.h>
#include <stdlib.h>

// TODO indicate error from bad input
int foobar(const char* s, int x) {
	int y = atoi(s);
	printf("s = %s\n", s);
	printf("x = %d\n", x);
	printf("y = %d\n", y);
	int result = x + y;
	printf("result = %d\n", result);
	return result;
}