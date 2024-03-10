#include<stdio.h>

int main() {
    FILE *f = fopen("/sys/class/power_supply/BAT1/voltage_now","r");
    double voltage;
    fscanf(f,"%lf",&voltage);
    fclose(f);
    f = fopen("/sys/class/power_supply/BAT1/current_now","r");
    double current;
    fscanf(f,"%lf",&current);
    printf("当前功耗：%.2lfW",current*voltage/1e12);
    return 0;
}