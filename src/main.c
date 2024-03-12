#include<stdio.h>
#include<dirent.h>
#include<stdlib.h>
#include<string.h>

#define BATTERY char*
BATTERY *batterys;

void find_batters() {
    int battery_count = 0;
    DIR *dir = opendir("/sys/class/power_supply");
    for (struct dirent *entry = readdir(dir); entry != NULL; entry = readdir(dir)) {
        if (entry->d_type == DT_DIR || entry->d_type == DT_LNK) {
            if (strncmp(entry->d_name,"BAT",3) == 0) {
                battery_count++;
                batterys = realloc(batterys,battery_count*sizeof(char*)+1);
                batterys[battery_count-1] = malloc(strlen(entry->d_name)+1);
                strcpy(batterys[battery_count-1],entry->d_name);
                batterys[battery_count] = NULL;
            }
        }
    }
    closedir(dir);
}

#define CHARGING 1
#define DISCHARGING 0
struct battery_info {
    double voltage;
    double current;
    int state;
};

struct battery_info get_battery_info(BATTERY battery) {
    struct battery_info info;
    char path[100];
    sprintf(path,"/sys/class/power_supply/%s/voltage_now",battery);
    FILE *f = fopen(path,"r");
    fscanf(f,"%lf",&info.voltage);
    fclose(f);
    sprintf(path,"/sys/class/power_supply/%s/current_now",battery);
    f = fopen(path,"r");
    fscanf(f,"%lf",&info.current);
    fclose(f);
    sprintf(path,"/sys/class/power_supply/%s/status",battery);
    f = fopen(path,"r");
    char status[20];
    fscanf(f,"%s",status);
    fclose(f);
    if (strcmp(status,"Charging") == 0) {
        info.state = CHARGING;
    } else {
        info.state = DISCHARGING;
    }

    return info;
}

void print_battery_info(BATTERY battery) {
    struct battery_info info = get_battery_info(battery);

    printf("电池：\t%s\n",battery);
    printf("状态：\t%s\n",info.state ? "充电" : "放电");
    printf("电压：\t%.2lfV\n",info.voltage/1e6);
    printf("电流：\t%.2lfA\n",info.current/1e6);
    printf("功率：\t%.2lfW\n",info.voltage*info.current/1e12);
}

int main() {
    find_batters();
    for (BATTERY *battery = batterys; *battery != NULL; battery++) {
        print_battery_info(*battery);
    }
    return 0;
}