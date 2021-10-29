# From https://github.com/stencila/examples/tree/master/elife-30274/sources

#Replication Study 48
#Study_48_Protocol_3/4
#R Version 3.4.1

#Required Packages
library(httr) #version 1.2.1
library(rjson) #version 0.2.15
library(ggplot2) #version 2.2.1
library(plyr) #version 1.8.4
library(Rmisc) #version 1.5
library(cowplot) #version 0.7.0
#source("~/credentials.R") #for private use during generation

#Downloads R script "download.OSF.file.R"
GET("https://osf.io/hkpjb/?action=download", write_disk("download.OSF.file.R", overwrite = TRUE))
source("download.OSF.file.R")
#calls the download.OSF.file

#Downloads data file 'Study_48_Protocols3&4_Combined_Means.csv' from https://osf.io/wmnuf/
download.OSF.file(GUID="wmnuf",Access_Token=RPCB_private_access,file_name="Study_48_Protocols3&4_Combined_Means.csv")

#reads csv file with all combined means for each lot
comb.means <- read.csv("Study_48_Protocols3&4_Combined_Means.csv", header=T, sep=",")

comb.means <- comb.means[which(comb.means$Status!="NA"),] #removes NA status genes
comb.means$lstat <- interaction(comb.means$Time, comb.means$Status) #creates interaction variable between lot and status called 'lstat'
comb.means$Time <- as.character(comb.means$Time) #creates a column for Time

active <- comb.means[which(comb.means$Status=="Active"),] #subsets all data on Active gene status
silent <- comb.means[which(comb.means$Status=="Silent"),] #subsets all data on Silent gene status

#create summary data for graph lot 1
active.sum1 <- summarySE(active[which(active$Lot=="C1"),], measurevar="final.mean", groupvars="Time")
silent.sum1 <- summarySE(silent[which(silent$Lot=="C1"),], measurevar="final.mean", groupvars="Time")

#create summary data for graph lot 2
active.sum2 <- summarySE(active[which(active$Lot=="C2"),], measurevar="final.mean", groupvars="Time")
silent.sum2 <- summarySE(silent[which(silent$Lot=="C2"),], measurevar="final.mean", groupvars="Time")

########## Plots Active Genes/ Lot 1 LOG SCALE ##########
#########################################################

log_activeplot1 <- ggplot(active[which(active$Lot=="C1"),], aes(x=Time, y = final.mean)) +
  stat_boxplot(geom ='errorbar', width=0.5) +
  geom_boxplot(aes(fill=Time), outlier.shape = 16, outlier.size = 1.5, outlier.colour = "black", colour = "black") +
  scale_fill_manual(values=c("red", "red", "red"))+
  ylab("Transcripts/Cell") +
  ggtitle("Active genes")+
  xlab(element_blank()) +
  scale_x_discrete(labels=c("0hr", "1hr", "24hr")) +
  scale_y_continuous(expand = c(.01,.01),
                     trans = "log2",
                     limits = c(2^-10,2^11),
                     breaks = c( 2^-9,2^-5,2^-1,2^3,2^7,2^11),
                     labels = c(bquote("2"^"-9"),bquote("2"^"-5"),
                                bquote("2"^"-1"),bquote("2"^"3"),
                                bquote("2"^"7"),bquote("2"^"11"))) +
  theme_bw()+
  theme(legend.position = "none",
        axis.ticks.length = unit(0.2, "cm"),
        plot.title = element_text(color = "black", size = 15, hjust = .5),
        plot.margin = unit(c(1,1,1,1),"cm"),
        panel.grid.major = element_blank(),
        panel.grid.minor = element_blank(),
        panel.background = element_rect(colour = "black", size=1.8),
        axis.title = element_text(colour = "black", size = 15),
        axis.text.x = element_text(colour="black",size=15, margin=margin(.5,.5,.5,.5)),
        axis.text.y = element_text(colour="black",size=15, margin=margin(.5,.5,.5,.5), hjust = 0),
        axis.title.x = element_blank(),
        axis.title.y = element_text(colour="black",size=15,margin = margin(r=10, unit="pt")))


########## Plots Active Genes/ Lot 1 LINEAR SCALE ##########
############################################################

linear_activeplot1 <- ggplot(active[which(active$Lot=="C1" & active$final.mean<=100),], aes(x=Time, y = final.mean)) +
  stat_boxplot(geom ='errorbar', width=0.5 ) +
  geom_boxplot(aes(fill=Time), outlier.shape = 16, outlier.size = 1.5, outlier.colour = "black", colour = "black") +
  scale_fill_manual(values=c("red", "red", "red"))+
  ggtitle("Active genes")+
  ylab("Transcripts/Cell") +
  xlab(element_blank()) +
  scale_x_discrete(labels=c("0hr", "1hr", "24hr")) +
  scale_y_continuous(expand = c(0,0),
                     limits = c(-5, 105),
                     breaks = c(0,20,40,60,80,100)) +
  theme_bw()+
  theme(legend.position = "none",
        axis.ticks.length = unit(0.2, "cm"),
        plot.title = element_text(color = "black", size = 15, hjust = .5),
        plot.margin = unit(c(1,1,1,1),"cm"),
        panel.grid.major = element_blank(),
        panel.grid.minor = element_blank(),
        panel.background = element_rect(colour = "black", size=1.8),
        axis.title = element_text(colour = "black", size = 15),
        axis.text.x = element_text(colour="black",size=15, margin=margin(.5,.5,.5,.5)),
        axis.text.y = element_text(colour="black",size=15, margin=margin(.5,.5,.5,.5), hjust = 0),
        axis.title.x = element_blank(),
        axis.title.y = element_text(colour="black",size=15,margin = margin(r=10, unit="pt")))


########## Plots Active Genes/ Lot 2 LOG SCALE ##########
#########################################################

log_activeplot2 <- ggplot(active[which(active$Lot=="C2"),], aes(x=Time, y = final.mean)) +
  stat_boxplot(geom ='errorbar', width=0.5 ) +
  geom_boxplot(aes(fill=Time), outlier.shape = 16, outlier.size = 1.5, outlier.colour = "black", colour = "black") +
  scale_fill_manual(values=c("red", "red", "red"))+
  ylab("Transcripts/Cell") +
  ggtitle("Active genes")+
  xlab(element_blank()) +
  scale_x_discrete(labels=c("0hr", "1hr", "24hr")) +
  scale_y_continuous(expand = c(.01,.01),
                     trans = "log2",
                     limits = c(2^-10,2^11),
                     breaks = c( 2^-9,2^-5,2^-1,2^3,2^7,2^11),
                     labels = c(bquote("2"^"-9"),bquote("2"^"-5"),
                                bquote("2"^"-1"),bquote("2"^"3"),
                                bquote("2"^"7"),bquote("2"^"11"))) +
  theme_bw()+
  theme(legend.position = "none",
        axis.ticks.length = unit(0.2, "cm"),
        plot.title = element_text(color = "black", size = 15, hjust = .5),
        plot.margin = unit(c(1,1,1,1),"cm"),
        panel.grid.major = element_blank(),
        panel.grid.minor = element_blank(),
        panel.background = element_rect(colour = "black", size=1.8),
        axis.title = element_text(colour = "black", size = 15),
        axis.text.x = element_text(colour="black",size=15, margin=margin(.5,.5,.5,.5)),
        axis.text.y = element_text(colour="black",size=15, margin=margin(.5,.5,.5,.5), hjust = 0),
        axis.title.x = element_blank(),
        axis.title.y = element_text(colour="black",size=15,margin = margin(r=10, unit="pt")))


########## Plots Active Genes/ Lot 2 LINEAR SCALE ##########
############################################################

linear_activeplot2 <- ggplot(active[which(active$Lot=="C2" & active$final.mean<=100),], aes(x=Time, y = final.mean)) +
  stat_boxplot(geom ='errorbar', width=0.5 ) +
  geom_boxplot(aes(fill=Time), outlier.shape = 16, outlier.size = 1.5, outlier.colour = "black", colour = "black") +
  scale_fill_manual(values=c("red", "red", "red"))+
  ggtitle("Active genes")+
  ylab("Transcripts/Cell") +
  xlab(element_blank()) +
  scale_x_discrete(labels=c("0hr", "1hr", "24hr")) +
  scale_y_continuous(expand = c(0,0),
                     limits = c(-5, 105),
                     breaks = c(0,20,40,60,80,100)) +
  theme_bw()+
  theme(legend.position = "none",
        axis.ticks.length = unit(0.2, "cm"),
        plot.title = element_text(color = "black", size = 15, hjust = .5),
        plot.margin = unit(c(1,1,1,1),"cm"),
        panel.grid.major = element_blank(),
        panel.grid.minor = element_blank(),
        panel.background = element_rect(colour = "black", size=1.8),
        axis.title = element_text(colour = "black", size = 15),
        axis.text.x = element_text(colour="black",size=15, margin=margin(.5,.5,.5,.5)),
        axis.text.y = element_text(colour="black",size=15, margin=margin(.5,.5,.5,.5), hjust = 0),
        axis.title.x = element_blank(),
        axis.title.y = element_text(colour="black",size=15,margin = margin(r=10, unit="pt")))


########## Plots Silent Genes/ Lot 1 LOG SCALE ##########
#########################################################

log_silentplot1 <- ggplot(silent[which(silent$Lot=="C1"),], aes(x=Time, y = final.mean)) +
  stat_boxplot(geom ='errorbar', width=0.5 ) +
  geom_boxplot(aes(fill=Time), outlier.shape = 16, outlier.size = 1.5, outlier.colour = "black", colour = "black") +
  scale_fill_manual(values=c("red", "red", "red"))+
  ylab("Transcripts/Cell") +
  ggtitle("Silent genes")+
  xlab(element_blank()) +
  scale_x_discrete(labels=c("0hr", "1hr", "24hr")) +
  scale_y_continuous(expand = c(.01,.01),
                     trans = "log2",
                     limits = c(2^-10,2^11),
                     breaks = c( 2^-9,2^-5,2^-1,2^3,2^7,2^11),
                     labels = c(bquote("2"^"-9"),bquote("2"^"-5"),
                                bquote("2"^"-1"),bquote("2"^"3"),
                                bquote("2"^"7"),bquote("2"^"11"))) +
  theme_bw()+
  theme(legend.position = "none",
        axis.ticks.length = unit(0.2, "cm"),
        plot.title = element_text(color = "black", size = 15, hjust = .5),
        plot.margin = unit(c(1,1,1,1),"cm"),
        panel.grid.major = element_blank(),
        panel.grid.minor = element_blank(),
        panel.background = element_rect(colour = "black", size=1.8),
        axis.title = element_text(colour = "black", size = 15),
        axis.text.x = element_text(colour="black",size=15, margin=margin(.5,.5,.5,.5)),
        axis.text.y = element_text(colour="black",size=15, margin=margin(.5,.5,.5,.5), hjust = 0),
        axis.title.x = element_blank(),
        axis.title.y = element_text(colour="black",size=15,margin = margin(r=10, unit="pt")))


########## Plots Silent Genes/ Lot 1 LINEAR SCALE ##########
############################################################

linear_silentplot1 <- ggplot(silent[which(silent$Lot=="C1" & silent$final.mean<=100),], aes(x=Time, y = final.mean)) +
  stat_boxplot(geom ='errorbar', width=0.5 ) +
  geom_boxplot(aes(fill=Time), outlier.shape = 16, outlier.size = 1.5, outlier.colour = "black", colour = "black") +
  scale_fill_manual(values=c("red", "red", "red"))+
  ggtitle("Silent genes")+
  ylab("Transcripts/Cell") +
  xlab(element_blank()) +
  scale_x_discrete(labels=c("0hr", "1hr", "24hr")) +
  scale_y_continuous(expand = c(0,0),
                     limits = c(-5, 105),
                     breaks = c(0,20,40,60,80,100)) +
  theme_bw()+
  theme(legend.position = "none",
        axis.ticks.length = unit(0.2, "cm"),
        plot.title = element_text(color = "black", size = 15, hjust = .5),
        plot.margin = unit(c(1,1,1,1),"cm"),
        panel.grid.major = element_blank(),
        panel.grid.minor = element_blank(),
        panel.background = element_rect(colour = "black", size=1.8),
        axis.title = element_text(colour = "black", size = 15),
        axis.text.x = element_text(colour="black",size=15, margin=margin(.5,.5,.5,.5)),
        axis.text.y = element_text(colour="black",size=15, margin=margin(.5,.5,.5,.5), hjust = 0),
        axis.title.x = element_blank(),
        axis.title.y = element_text(colour="black",size=15,margin = margin(r=10, unit="pt")))


########## Plots Silent Genes/ Lot 2 LOG SCALE ##########
#########################################################

#plots active cohort 1
log_silentplot2 <- ggplot(silent[which(silent$Lot=="C2"),], aes(x=Time, y = final.mean)) +
  stat_boxplot(geom ='errorbar', width=0.5 ) +
  geom_boxplot(aes(fill=Time), outlier.shape = 16, outlier.size = 1.5, outlier.colour = "black", colour = "black") +
  scale_fill_manual(values=c("red", "red", "red"))+
  ylab("Transcripts/Cell") +
  ggtitle("Silent genes")+
  xlab(element_blank()) +
  scale_x_discrete(labels=c("0hr", "1hr", "24hr")) +
  scale_y_continuous(expand = c(.01,.01),
                     trans = "log2",
                     limits = c(2^-10,2^11),
                     breaks = c( 2^-9,2^-5,2^-1,2^3,2^7,2^11),
                     labels = c(bquote("2"^"-9"),bquote("2"^"-5"),
                                bquote("2"^"-1"),bquote("2"^"3"),
                                bquote("2"^"7"),bquote("2"^"11"))) +
  theme_bw()+
  theme(legend.position = "none",
        axis.ticks.length = unit(0.2, "cm"),
        plot.title = element_text(color = "black", size = 15, hjust = .5),
        plot.margin = unit(c(1,1,1,1),"cm"),
        panel.grid.major = element_blank(),
        panel.grid.minor = element_blank(),
        panel.background = element_rect(colour = "black", size=1.8),
        axis.title = element_text(colour = "black", size = 15),
        axis.text.x = element_text(colour="black",size=15, margin=margin(.5,.5,.5,.5)),
        axis.text.y = element_text(colour="black",size=15, margin=margin(.5,.5,.5,.5), hjust = 0),
        axis.title.x = element_blank(),
        axis.title.y = element_text(colour="black",size=15,margin = margin(r=10, unit="pt")))


########## Plots Silent Genes/ Lot 2 LINEAR SCALE ##########
############################################################

linear_silentplot2 <- ggplot(silent[which(silent$Lot=="C2" & silent$final.mean<=100),], aes(x=Time, y = final.mean)) +
  stat_boxplot(geom ='errorbar', width=0.5 ) +
  geom_boxplot(aes(fill=Time), outlier.shape = 16, outlier.size = 1.5, outlier.colour = "black", colour = "black") +
  scale_fill_manual(values=c("red", "red", "red")) +
  ggtitle("Silent genes")+
  ylab("Transcripts/Cell") +
  xlab(element_blank()) +
  scale_x_discrete(labels=c("0hr", "1hr", "24hr")) +
  scale_y_continuous(expand = c(0,0),
                     limits = c(-5, 105),
                     breaks = c(0,20,40,60,80,100)) +
  theme_bw()+
  theme(legend.position = "none",
        axis.ticks.length = unit(0.2, "cm"),
        plot.title = element_text(color = "black", size = 15, hjust = .5),
        plot.margin = unit(c(1,1,1,1.88),"cm"),
        panel.grid.major = element_blank(),
        panel.grid.minor = element_blank(),
        panel.background = element_rect(colour = "black", size=1.8),
        axis.title = element_text(colour = "black", size = 15),
        axis.text.x = element_text(colour="black",size=15, margin=margin(.5,.5,.5,.5)),
        axis.text.y = element_text(colour="black",size=15, margin=margin(.5,.5,.5,.5), hjust = 0),
        axis.title.x = element_blank(),
        axis.title.y = element_text(colour="black",size=15,margin = margin(r=10, unit="pt")))

###########################################################################################
###########################################################################################

#plots all comparisons for Lot 1 silent
lot1_silent <- subset(comb.means, comb.means$Lot=="C1" & comb.means$Status=="Silent")
time0 <- subset(lot1_silent, lot1_silent$Time=="0HR")
time1 <- subset(lot1_silent, lot1_silent$Time=="1HR")
time24 <- subset(lot1_silent, lot1_silent$Time=="24HR")
ratio <- c(((log2(time1$final.mean))-(log2(time0$final.mean))),
           ((log2(time24$final.mean))-(log2(time0$final.mean))),
           ((log2(time24$final.mean))-(log2(time1$final.mean))))
lot1_silentdat <- as.data.frame(cbind(as.character(lot1_silent[,1]),as.numeric(as.character(ratio))))
lot1_silentdat$V1 <- as.factor(lot1_silentdat$V1)
lot1_silentdat$V3 <- c(rep("diff1",nrow(lot1_silent)/3),rep("diff2",nrow(lot1_silent)/3),rep("diff3",nrow(lot1_silent)/3))
lot1_silentdat$V3 <- as.factor(lot1_silentdat$V3)
lot1_silentdat$V2 <- as.numeric(as.character(lot1_silentdat$V2))
colnames(lot1_silentdat) <- c("Gene","ratio","comparison")

plot_lot1_silent <- ggplot(lot1_silentdat, aes(x=comparison, y = ratio)) +
  stat_boxplot(geom ='errorbar', width=0.5) +
  geom_boxplot(aes(fill=comparison), outlier.shape = 16, outlier.size = 1.5, outlier.colour = "black", colour = "black") +
  scale_fill_manual(values=c("red", "red", "red"))+
  ylab("log2 (ratio)") +
  xlab(element_blank()) +
  scale_x_discrete(labels=c("1 hr vs. \n 0 hr", "24 hr vs. \n 0 hr", "24hr vs. \n 1 hr")) +
  scale_y_continuous(expand = c(0,0),
                     limits = c(-4.5,6.5),
                     breaks = c(-4, -2, 0, 2, 4, 6),
                     labels = c("-4","-2","0","2","4","6")) +
  geom_hline(yintercept = 0) +
  ggtitle("Silent genes")+
  theme_bw()+
  theme(legend.position = "none",
        axis.ticks.length = unit(0.2, "cm"),
        plot.title = element_text(color = "black", size = 15, hjust = .5),
        plot.margin = unit(c(1,1,1,1),"cm"),
        panel.grid.major = element_blank(),
        panel.grid.minor = element_blank(),
        panel.background = element_rect(colour = "black", size=1.8),
        axis.title = element_text(colour = "black", size = 15),
        axis.text.x = element_text(colour="black",size=15, margin=margin(.5,.5,.5,.5)),
        axis.text.y = element_text(colour="black",size=15, margin=margin(.5,.5,.5,.5)),
        axis.title.x = element_blank(),
        axis.title.y = element_text(colour="black",size=15,margin = margin(r=10, unit="pt")))
plot_lot1_silent

####################################################
####################################################

#plots all comparisons for Lot 2 silent
lot2_silent <- subset(comb.means, comb.means$Lot=="C2" & comb.means$Status=="Silent")
time0 <- subset(lot2_silent, lot2_silent$Time=="0HR")
time1 <- subset(lot2_silent, lot2_silent$Time=="1HR")
time24 <- subset(lot2_silent, lot2_silent$Time=="24HR")
ratio <- c(((log2(time1$final.mean))-(log2(time0$final.mean))),
           ((log2(time24$final.mean))-(log2(time0$final.mean))),
           ((log2(time24$final.mean))-(log2(time1$final.mean))))
lot2_silentdat <- as.data.frame(cbind(as.character(lot2_silent[,1]),as.numeric(as.character(ratio))))
lot2_silentdat$V1 <- as.factor(lot2_silentdat$V1)
lot2_silentdat$V3 <- c(rep("diff1",nrow(lot2_silent)/3),rep("diff2",nrow(lot2_silent)/3),rep("diff3",nrow(lot2_silent)/3))
lot2_silentdat$V3 <- as.factor(lot2_silentdat$V3)
lot2_silentdat$V2 <- as.numeric(as.character(lot2_silentdat$V2))
colnames(lot2_silentdat) <- c("Gene","ratio","comparison")

plot_lot2_silent <- ggplot(lot2_silentdat, aes(x=comparison, y = ratio)) +
  stat_boxplot(geom ='errorbar', width=0.5) +
  geom_boxplot(aes(fill=comparison), outlier.shape = 16, outlier.size = 1.5, outlier.colour = "black", colour = "black") +
  scale_fill_manual(values=c("red", "red", "red"))+
  ylab("log2 (ratio)") +
  xlab(element_blank()) +
  scale_x_discrete(labels=c("1 hr vs. \n 0 hr", "24 hr vs. \n 0 hr", "24hr vs. \n 1 hr")) +
  scale_y_continuous(expand = c(0,0),
                     limits = c(-4.5,6.5),
                     breaks = c(-4, -2, 0, 2, 4, 6),
                     labels = c("-4","-2","0","2","4","6")) +
  geom_hline(yintercept = 0) +
  ggtitle("Silent genes")+
  theme_bw()+
  theme(legend.position = "none",
        axis.ticks.length = unit(0.2, "cm"),
        plot.title = element_text(color = "black", size = 15, hjust = .5),
        plot.margin = unit(c(1,1,1,1),"cm"),
        panel.grid.major = element_blank(),
        panel.grid.minor = element_blank(),
        panel.background = element_rect(colour = "black", size=1.8),
        axis.title = element_text(colour = "black", size = 15),
        axis.text.x = element_text(colour="black",size=15, margin=margin(.5,.5,.5,.5)),
        axis.text.y = element_text(colour="black",size=15, margin=margin(.5,.5,.5,.5)),
        axis.title.x = element_blank(),
        axis.title.y = element_text(colour="black",size=15,margin = margin(r=10, unit="pt")))
plot_lot2_silent

####################################################
####################################################

#plots all comparisons for Lot 1 active
lot1_active <- subset(comb.means, comb.means$Lot=="C1" & comb.means$Status=="Active")
time0 <- subset(lot1_active, lot1_active$Time=="0HR")
time1 <- subset(lot1_active, lot1_active$Time=="1HR")
time24 <- subset(lot1_active, lot1_active$Time=="24HR")
ratio <- c(((log2(time1$final.mean))-(log2(time0$final.mean))),
           ((log2(time24$final.mean))-(log2(time0$final.mean))),
           ((log2(time24$final.mean))-(log2(time1$final.mean))))
lot1_activedat <- as.data.frame(cbind(as.character(lot1_active[,1]),as.numeric(as.character(ratio))))
lot1_activedat$V1 <- as.factor(lot1_activedat$V1)
lot1_activedat$V3 <- c(rep("diff1",nrow(lot1_active)/3),rep("diff2",nrow(lot1_active)/3),rep("diff3",nrow(lot1_active)/3))
lot1_activedat$V3 <- as.factor(lot1_activedat$V3)
lot1_activedat$V2 <- as.numeric(as.character(lot1_activedat$V2))
colnames(lot1_activedat) <- c("Gene","ratio","comparison")

plot_lot1_active <- ggplot(lot1_activedat, aes(x=comparison, y = ratio)) +
  stat_boxplot(geom ='errorbar', width=0.5) +
  geom_boxplot(aes(fill=comparison), outlier.shape = 16, outlier.size = 1.5, outlier.colour = "black", colour = "black") +
  scale_fill_manual(values=c("red", "red", "red"))+
  ylab("log2 (ratio)") +
  xlab(element_blank()) +
  scale_x_discrete(labels=c("1 hr vs. \n 0 hr", "24 hr vs. \n 0 hr", "24hr vs. \n 1 hr")) +
  scale_y_continuous(expand = c(0,0),
                     limits = c(-4.5,6.5),
                     breaks = c(-4, -2, 0, 2, 4, 6),
                     labels = c("-4","-2","0","2","4","6")) +
  geom_hline(yintercept = 0) +
  ggtitle("Active genes")+
  theme_bw()+
  theme(legend.position = "none",
        axis.ticks.length = unit(0.2, "cm"),
        plot.title = element_text(color = "black", size = 15, hjust = .5),
        plot.margin = unit(c(1,1,1,1),"cm"),
        panel.grid.major = element_blank(),
        panel.grid.minor = element_blank(),
        panel.background = element_rect(colour = "black", size=1.8),
        axis.title = element_text(colour = "black", size = 15),
        axis.text.x = element_text(colour="black",size=15, margin=margin(.5,.5,.5,.5)),
        axis.text.y = element_text(colour="black",size=15, margin=margin(.5,.5,.5,.5)),
        axis.title.x = element_blank(),
        axis.title.y = element_text(colour="black",size=15,margin = margin(r=10, unit="pt")))
plot_lot1_active

####################################################
####################################################

#plots all comparisons for Lot 2 active
lot2_active <- subset(comb.means, comb.means$Lot=="C2" & comb.means$Status=="Active")
time0 <- subset(lot2_active, lot2_active$Time=="0HR")
time1 <- subset(lot2_active, lot2_active$Time=="1HR")
time24 <- subset(lot2_active, lot2_active$Time=="24HR")
ratio <- c(((log2(time1$final.mean))-(log2(time0$final.mean))),
           ((log2(time24$final.mean))-(log2(time0$final.mean))),
           ((log2(time24$final.mean))-(log2(time1$final.mean))))
lot2_activedat <- as.data.frame(cbind(as.character(lot2_active[,1]),as.numeric(as.character(ratio))))
lot2_activedat$V1 <- as.factor(lot2_activedat$V1)
lot2_activedat$V3 <- c(rep("diff1",nrow(lot2_active)/3),rep("diff2",nrow(lot2_active)/3),rep("diff3",nrow(lot2_active)/3))
lot2_activedat$V3 <- as.factor(lot2_activedat$V3)
lot2_activedat$V2 <- as.numeric(as.character(lot2_activedat$V2))
colnames(lot2_activedat) <- c("Gene","ratio","comparison")

plot_lot2_active <- ggplot(lot2_activedat, aes(x=comparison, y = ratio)) +
  stat_boxplot(geom ='errorbar', width=0.5) +
  geom_boxplot(aes(fill=comparison), outlier.shape = 16, outlier.size = 1.5, outlier.colour = "black", colour = "black") +
  scale_fill_manual(values=c("red", "red", "red"))+
  ylab("log2 (ratio)") +
  xlab(element_blank()) +
  scale_x_discrete(labels=c("1 hr vs. \n 0 hr", "24 hr vs. \n 0 hr", "24hr vs. \n 1 hr")) +
  scale_y_continuous(expand = c(0,0),
                     limits = c(-4.5,6.5),
                     breaks = c(-4, -2, 0, 2, 4, 6),
                     labels = c("-4","-2","0","2","4","6")) +
  geom_hline(yintercept = 0) +
  ggtitle("Active genes")+
  theme_bw()+
  theme(legend.position = "none",
        axis.ticks.length = unit(0.2, "cm"),
        plot.title = element_text(color = "black", size = 15, hjust = .5),
        plot.margin = unit(c(1,1,1,1),"cm"),
        panel.grid.major = element_blank(),
        panel.grid.minor = element_blank(),
        panel.background = element_rect(colour = "black", size=1.8),
        axis.title = element_text(colour = "black", size = 15),
        axis.text.x = element_text(colour="black",size=15, margin=margin(.5,.5,.5,.5)),
        axis.text.y = element_text(colour="black",size=15, margin=margin(.5,.5,.5,.5)),
        axis.title.x = element_blank(),
        axis.title.y = element_text(colour="black",size=15, margin = margin(r=10, unit="pt")))
plot_lot2_active

###########################################################################################
###########################################################################################

#combines lot 1 linear scale plots
linear_lot1 <- plot_grid(linear_activeplot1, linear_silentplot1, ncol = 2, align = "h")
#combines lot 2 linear scale plots
linear_lot2 <- plot_grid(linear_activeplot2, linear_silentplot2, ncol = 2, align = "h")

#combines lot 1 log scale plots
log_lot1 <- plot_grid(log_activeplot1, log_silentplot1, ncol = 2, align = "h", labels = c("A","B"))
#combines lot 2 log scale plots
log_lot2 <- plot_grid(log_activeplot2, log_silentplot2, ncol = 2, align = "h", labels = c("E","F"))

#combines lot 1 ratio plots
ratio_lot1 <- plot_grid(plot_lot1_active, plot_lot1_silent, ncol = 2, align = "h", labels = c("C","D"))
#combines lot 2 ratio plots
ratio_lot2 <- plot_grid(plot_lot2_active, plot_lot2_silent, ncol = 2, align = "h", labels = c("G","H"))

#combines Linear Scale Plots
Linear <- plot_grid(linear_lot1,linear_lot2, nrow = 2, align = "h", labels = c("Lot 1","Lot 2"), label_size = 20)
title <- ggdraw() + draw_label("Figure 2", size=20)
figure_2 <- plot_grid(title,Linear,ncol = 1,rel_heights = c(0.1,1))
figure_2
# saves file 'Study_48_Figure_2.pdf' locally
ggsave(file = "Study_48_Figure _2.pdf", width = 8, height = 12)

#combines Ratio Plots
Ratio_Log <- plot_grid(log_lot1,  ratio_lot1, log_lot2, ratio_lot2, ncol = 1, align = "h")
title <- ggdraw() + draw_label("Figure 2 - figure supplement 1", size=20)
figure_2_s1 <- plot_grid(title,Ratio_Log,ncol = 1,rel_heights = c(0.1,1))
figure_2_s1
# saves file 'Study_48_Figure_2_figure_supplement_1.pdf' locally
ggsave(file = "Study_48_Figure_2_figure_supplement_1.pdf", width = 10, height = 18)
