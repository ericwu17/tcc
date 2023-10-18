int main() {
    for (int i = 0; i < 1000; i ++) {
        int isPrime = 1;
        int factor = 2;
        while (factor < i) {
            if (i % factor == 0) {
                isPrime = 0;
                break;
            }
            factor++;
        }

        if (isPrime) {
            // print out the prime
            int temp = i;

            // count the digits since we will need to print most significant digit first
            int numDigits = 0;
            while (temp > 0) {
                temp = temp / 10;
                numDigits ++;
            }

            // print each digit
            for (int digit = 0; digit < numDigits; digit ++) {
                int temp  = i;
                for (int i = 1; i < numDigits - digit; i ++) {
                    temp = temp / 10;
                }
                putchar(temp % 10 + '0');
            }

            putchar('\n');
            
        }
    }
}
