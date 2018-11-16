library(ggplot2)
library(cowplot)

#reads csv file
meta <- read.csv("article/Study_48_Meta_Analysis.csv", header = T)

#######################################
#re-orders the data frame
meta <- meta[order(meta$study),]
meta <- meta[order(meta$comparison),]

#subsets data to plot results
d1 <- subset(meta[1:4,]) #Active 0v1
d2 <- subset(meta[5:8,]) #Active 0v24
d3 <- subset(meta[9:12,]) #Active 1v24

d4 <- subset(meta[13:16,]) #Silent 0v1
d5 <- subset(meta[17:20,]) #Silent 0v24
d6 <- subset(meta[21:24,]) #Silent 1v24

d7 <- subset(meta[25:28,]) #protocol 2 oh vs. 24hr

#Plots for protocol 3 analyses:
########################### Active 0hr vs. 1hr #####################################

#re-order the levels for plotting
desired_order <- c("Meta-Analysis","RP:CB Lot 2","RP:CB Lot 1", "Lin et al., 2012" )

#re-orders data for plotting
d1$study <- factor(as.character(d1$study), levels=desired_order)
d1 <- d1[order(d1$study),]

a1 <- ggplot(data=d1,aes(x=estimate,y=d1$study)) +
  geom_point(size=5, colour="black", fill = "black", shape = c(22,21,21,23)) +  
  geom_errorbarh(aes(xmin=CI.lower,xmax=CI.upper, height = .1)) +
  geom_vline(xintercept=0,linetype="dashed") +
  coord_cartesian(xlim=c(-.2,1)) +
  scale_x_continuous(breaks= c(-.2,0,.2,.4,.6,.8,1)) +
  ylab(d1$comparison[1])+
  xlab(NULL)+
  scale_y_discrete(labels = c(paste(as.character(d1$study[1])),
                              gsub("x", "\n",paste(as.character(d1$study[2]),
                                                   paste("x (n =", as.character(d1$N[2]),")"))),
                              gsub("x", "\n",paste(as.character(d1$study[3]),
                                                   paste("x (n =", as.character(d1$N[3]),")"))),
                              gsub("x", "\n",paste(as.character(d1$study[4]),
                                                   paste("x (n =", as.character(d1$N[4]),")"))))) +
  theme(panel.background = element_blank(),
        axis.ticks.x=element_blank(), 
        axis.title.x = element_blank(), 
        axis.text.x = element_blank(),
        axis.text.y = element_text(size=15),
        legend.position="none",
        axis.line.x = element_blank(),
        axis.title.y = element_text(size=15,margin=margin(0,30,0,0)),
        axis.line.y = element_line(),
        plot.margin = unit(c(.5 ,2,.075,.5), "in"))
a1 <- ggdraw(a1)+
  draw_text(paste("r","[","L.CI", ",", "U.CI", "]"), x = .84, y = 0.91, fontface="bold")+
  draw_text(paste(formatC(d1$estimate[[4]], digits =  2, format = "f"),
                  "[",
                  formatC(d1$CI.lower[[4]],digits = 2, format = "f"),
                  ",", 
                  formatC(d1$CI.upper[[4]],digits = 2, format = "f"),
                  "]"), x = 0.75, y = 0.785, size = 12, hjust=0)+
  draw_text(paste(formatC(d1$estimate[[3]], digits = 2, format = "f"),
                  "[",
                  formatC(d1$CI.lower[[3]],digits = 2, format = "f"),
                  ",", 
                  formatC(d1$CI.upper[[3]],digits = 2, format = "f"),
                  "]"), x = 0.75, y = 0.575, size = 12, hjust=0)+
  draw_text(paste(formatC(d1$estimate[[2]],digits = 2, format = "f"),
                  "[",
                  formatC(d1$CI.lower[[2]],digits = 2, format = "f"),
                  ",", 
                  formatC(d1$CI.upper[[2]],digits = 2, format = "f"),
                  "]"), x = 0.75, y =  0.3575, size = 12, hjust=0)+
  draw_text(paste(formatC(d1$estimate[[1]],digits = 2, format = "f"),
                  "[",
                  formatC(d1$CI.lower[[1]],digits = 2, format = "f"),
                  ",", 
                  formatC(d1$CI.upper[[1]],digits = 2, format = "f"),
                  "]"), x = 0.75, y = 0.154, size = 12, hjust=0)

########################## Active 0hr vs. 24hr #####################################

#re-order the levels for plotting
d2$study <- factor(as.character(d2$study), levels=desired_order)
d2 <- d2[order(d2$study),]

a2 <- ggplot(data=d2,aes(x=estimate,y=d2$study)) +
  geom_point(size=5, colour="black", fill = "black", shape = c(22,21,21,23)) +  
  geom_errorbarh(aes(xmin=CI.lower,xmax=CI.upper, height = .1)) +
  geom_vline(xintercept=0,linetype="dashed") +
  coord_cartesian(xlim=c(-.2,1)) +
  scale_x_continuous(breaks= c(-.2,0,.2,.4,.6,.8,1)) +
  ylab(d2$comparison[1])+
  xlab(NULL)+
  scale_y_discrete(labels = c(paste(as.character(d2$study[1])),
                              gsub("x", "\n",paste(as.character(d2$study[2]),
                                                   paste("x (n =", as.character(d2$N[2]),")"))),
                              gsub("x", "\n",paste(as.character(d2$study[3]),
                                                   paste("x (n =", as.character(d2$N[3]),")"))),
                              gsub("x", "\n",paste(as.character(d2$study[4]),
                                                   paste("x (n =", as.character(d2$N[4]),")")))))+
  theme(panel.background = element_blank(),
        axis.ticks.x=element_blank(), 
        axis.title.x = element_blank(), 
        axis.text.x = element_blank(),
        axis.text.y = element_text(size=15),
        legend.position="none",
        axis.line.x = element_blank(),
        axis.title.y = element_text(size=15,margin=margin(0,30,0,0)),
        axis.line.y = element_line(),
        plot.margin = unit(c(.22,2,.22,.5), "in"))
a2 <- ggdraw(a2)+
  draw_text(paste(formatC(d2$estimate[[4]], digits =  2, format = "f"),
                  "[",
                  formatC(d2$CI.lower[[4]],digits = 2, format = "f"),
                  ",", 
                  formatC(d2$CI.upper[[4]],digits = 2, format = "f"),
                  "]"), x = 0.75, y = 0.829, size = 12, hjust=0)+
  draw_text(paste(formatC(d2$estimate[[3]], digits = 2, format = "f"),
                  "[",
                  formatC(d2$CI.lower[[3]],digits = 2, format = "f"),
                  ",", 
                  formatC(d2$CI.upper[[3]],digits = 2, format = "f"),
                  "]"), x = 0.75, y = 0.615, size = 12, hjust=0)+
  draw_text(paste(formatC(d2$estimate[[2]],digits = 2, format = "f"),
                  "[",
                  formatC(d2$CI.lower[[2]],digits = 2, format = "f"),
                  ",", 
                  formatC(d2$CI.upper[[2]],digits = 2, format = "f"),
                  "]"), x = 0.75, y =  0.395, size = 12, hjust=0)+
  draw_text(paste(formatC(d2$estimate[[1]],digits = 2, format = "f"),
                  "[",
                  formatC(d2$CI.lower[[1]],digits = 2, format = "f"),
                  ",", 
                  formatC(d2$CI.upper[[1]],digits = 2, format = "f"),
                  "]"), x = 0.75, y = 0.183, size = 12, hjust=0)

########################### Active 1hr vs. 24hr #####################################

#re-order the levels for plotting
d3$study <- factor(as.character(d3$study), levels=desired_order)
d3 <- d3[order(d3$study),]

a3 <- ggplot(data=d3,aes(x=estimate,y=d3$study))+
  geom_point(size=5, colour="black", fill = "black", shape = c(22,21,21,23)) +  
  geom_errorbarh(aes(xmin=CI.lower,xmax=CI.upper, height = .1)) +
  geom_vline(xintercept=0,linetype="dashed") +
  coord_cartesian(xlim=c(-.2,1)) +
  scale_x_continuous(breaks= c(-.2,0,.2,.4,.6,.8,1)) +
  ylab(d3$comparison[1]) +
  xlab("r") +
  scale_y_discrete(labels = c(paste(as.character(d3$study[1])),
                              gsub("x", "\n",paste(as.character(d3$study[2]),
                                                   paste("x (n =", as.character(d3$N[2]),")"))),
                              gsub("x", "\n",paste(as.character(d3$study[3]),
                                                   paste("x (n =", as.character(d3$N[3]),")"))),
                              gsub("x", "\n",paste(as.character(d3$study[4]),
                                                   paste("x (n =", as.character(d3$N[4]),")"))))) +
  theme(panel.background = element_blank(),
        legend.position="none",
        axis.line.x = element_line(),
        axis.title.y = element_text(size=15,margin=margin(0,30,0,0)),
        axis.text.y = element_text(size=15),
        axis.title.x = element_text(size=15),
        axis.line.y = element_line(),
        plot.margin = unit(c(.1, 2,0,.5), "in"))
a3 <- ggdraw(a3)+
  draw_text(paste(formatC(d3$estimate[[4]], digits =  2, format = "f"),
                  "[",
                  formatC(d3$CI.lower[[4]],digits = 2, format = "f"),
                  ",", 
                  formatC(d3$CI.upper[[4]],digits = 2, format = "f"),
                  "]"), x = 0.75, y = 0.858, size = 12, hjust=0)+
  draw_text(paste(formatC(d3$estimate[[3]], digits = 2, format = "f"),
                  "[",
                  formatC(d3$CI.lower[[3]],digits = 2, format = "f"),
                  ",", 
                  formatC(d3$CI.upper[[3]],digits = 2, format = "f"),
                  "]"), x = 0.75, y = 0.6445, size = 12, hjust=0)+
  draw_text(paste(formatC(d3$estimate[[2]],digits = 2, format = "f"),
                  "[",
                  formatC(d3$CI.lower[[2]],digits = 2, format = "f"),
                  ",", 
                  formatC(d3$CI.upper[[2]],digits = 2, format = "f"),
                  "]"), x = 0.75, y =  0.432, size = 12, hjust=0)+
  draw_text(paste(formatC(d3$estimate[[1]],digits = 2, format = "f"),
                  "[",
                  formatC(d3$CI.lower[[1]],digits = 2, format = "f"),
                  ",", 
                  formatC(d3$CI.upper[[1]],digits = 2, format = "f"),
                  "]"), x = 0.75, y = 0.218, size = 12, hjust=0)

#Plots for protocol 3 analyses Silent genes:
########################### Silent 0hr vs. 1hr #####################################

#re-order the levels for plotting
d4$study <- factor(as.character(d4$study), levels=desired_order)
d4 <- d4[order(d4$study),]

s1 <- ggplot(data=d4,aes(x=estimate,y=d4$study)) +
  geom_point(size=5, colour="black", fill = "black", shape = c(22,21,21,23)) +  
  geom_errorbarh(aes(xmin=CI.lower,xmax=CI.upper, height = .1)) +
  geom_vline(xintercept=0,linetype="dashed") +
  coord_cartesian(xlim=c(-.2,1)) +
  scale_x_continuous(breaks= c(-.2,0,.2,.4,.6,.8,1)) +
  ylab(d4$comparison[1])+
  xlab(NULL)+
  scale_y_discrete(labels = c(paste(as.character(d4$study[1])),
                              gsub("x", "\n",paste(as.character(d4$study[2]),
                                                   paste("x (n =", as.character(d4$N[2]),")"))),
                              gsub("x", "\n",paste(as.character(d4$study[3]),
                                                   paste("x (n =", as.character(d4$N[3]),")"))),
                              gsub("x", "\n",paste(as.character(d4$study[4]),
                                                   paste("x (n =", as.character(d4$N[4]),")")))))+
  theme(panel.background = element_blank(),
        axis.ticks.x=element_blank(), 
        axis.title.x = element_blank(), 
        axis.text.x = element_blank(),
        axis.text.y = element_text(size=15),
        legend.position="none",
        axis.line.x = element_blank(),
        axis.title.y = element_text(size=15,margin=margin(0,30,0,0)),
        axis.line.y = element_line(),
        plot.margin = unit(c(.5 ,2,.075,.5), "in"))
s1 <- ggdraw(s1)+
  draw_text(paste("r","[","L.CI", ",", "U.CI", "]"), x = .84, y = 0.91, fontface="bold")+
  draw_text(paste(formatC(d4$estimate[[4]], digits =  2, format = "f"),
                  "[",
                  formatC(d4$CI.lower[[4]],digits = 2, format = "f"),
                  ",", 
                  formatC(d4$CI.upper[[4]],digits = 2, format = "f"),
                  "]"), x = 0.75, y = 0.785, size = 12, hjust=0)+
  draw_text(paste(formatC(d4$estimate[[3]], digits = 2, format = "f"),
                  "[",
                  formatC(d4$CI.lower[[3]],digits = 2, format = "f"),
                  ",", 
                  formatC(d4$CI.upper[[3]],digits = 2, format = "f"),
                  "]"), x = 0.75, y = 0.575, size = 12, hjust=0)+
  draw_text(paste(formatC(d4$estimate[[2]],digits = 2, format = "f"),
                  "[",
                  formatC(d4$CI.lower[[2]],digits = 2, format = "f"),
                  ",", 
                  formatC(d4$CI.upper[[2]],digits = 2, format = "f"),
                  "]"), x = 0.75, y =  0.3575, size = 12, hjust=0)+
  draw_text(paste(formatC(d4$estimate[[1]],digits = 2, format = "f"),
                  "[",
                  formatC(d4$CI.lower[[1]],digits = 2, format = "f"),
                  ",", 
                  formatC(d4$CI.upper[[1]],digits = 2, format = "f"),
                  "]"), x = 0.75, y = 0.154, size = 12, hjust=0)


########################## Silent 0hr vs. 24hr #####################################

#re-order the levels for plotting
d5$study <- factor(as.character(d5$study), levels=desired_order)
d5 <- d5[order(d5$study),]

s2 <- ggplot(data=d5,aes(x=estimate,y=d5$study)) +
  geom_point(size=5, colour="black", fill = "black", shape = c(22,21,21,23)) +  
  geom_errorbarh(aes(xmin=CI.lower,xmax=CI.upper, height = .1)) +
  geom_vline(xintercept=0,linetype="dashed") +
  coord_cartesian(xlim=c(-.2,1)) +
  scale_x_continuous(breaks= c(-.2,0,.2,.4,.6,.8,1)) +
  ylab(d5$comparison[1])+
  xlab(NULL)+
  scale_y_discrete(labels = c(paste(as.character(d5$study[1])),
                              gsub("x", "\n",paste(as.character(d5$study[2]),
                                                   paste("x (n =", as.character(d5$N[2]),")"))),
                              gsub("x", "\n",paste(as.character(d5$study[3]),
                                                   paste("x (n =", as.character(d5$N[3]),")"))),
                              gsub("x", "\n",paste(as.character(d5$study[4]),
                                                   paste("x (n =", as.character(d5$N[4]),")")))))+
  theme(panel.background = element_blank(),
        axis.ticks.x=element_blank(), 
        axis.title.x = element_blank(), 
        axis.text.x = element_blank(),
        axis.text.y = element_text(size=15),
        legend.position="none",
        axis.line.x = element_blank(),
        axis.title.y = element_text(size=15,margin=margin(0,30,0,0)),
        axis.line.y = element_line(),
        plot.margin = unit(c(.22,2,.22,.5), "in"))
s2 <- ggdraw(s2)+
  draw_text(paste(formatC(d5$estimate[[4]], digits =  2, format = "f"),
                  "[",
                  formatC(d5$CI.lower[[4]],digits = 2, format = "f"),
                  ",", 
                  formatC(d5$CI.upper[[4]],digits = 2, format = "f"),
                  "]"), x = 0.75, y = 0.829, size = 12, hjust=0)+
  draw_text(paste(formatC(d5$estimate[[3]], digits = 2, format = "f"),
                  "[",
                  formatC(d5$CI.lower[[3]],digits = 2, format = "f"),
                  ",", 
                  formatC(d5$CI.upper[[3]],digits = 2, format = "f"),
                  "]"), x = 0.75, y = 0.615, size = 12, hjust=0)+
  draw_text(paste(formatC(d5$estimate[[2]],digits = 2, format = "f"),
                  "[",
                  formatC(d5$CI.lower[[2]],digits = 2, format = "f"),
                  ",", 
                  formatC(d5$CI.upper[[2]],digits = 2, format = "f"),
                  "]"), x = 0.75, y =  0.395, size = 12, hjust=0)+
  draw_text(paste(formatC(d5$estimate[[1]],digits = 2, format = "f"),
                  "[",
                  formatC(d5$CI.lower[[1]],digits = 2, format = "f"),
                  ",", 
                  formatC(d5$CI.upper[[1]],digits = 2, format = "f"),
                  "]"), x = 0.75, y = 0.183, size = 12, hjust=0)

########################## Silent 1hr vs. 24hr #####################################

#re-order the levels for plotting
d6$study <- factor(as.character(d6$study), levels=desired_order)
d6 <- d6[order(d6$study),]

s3 <- ggplot(data=d6,aes(x=estimate,y=factor(d6$study)))+
  geom_point(size=5, colour="black", fill = "black", shape = c(22,21,21,23)) +  
  geom_errorbarh(aes(xmin=CI.lower,xmax=CI.upper, height = .1)) +
  geom_vline(xintercept=0,linetype="dashed") +
  coord_cartesian(xlim=c(-.2,1)) +
  scale_x_continuous(breaks= c(-.2,0,.2,.4,.6,.8,1)) +
  ylab(d6$comparison[1]) +
  xlab("r") +
  scale_y_discrete(labels = c(paste(as.character(d6$study[1])),
                              gsub("x", "\n",paste(as.character(d6$study[2]),
                                                   paste("x (n =", as.character(d6$N[2]),")"))),
                              gsub("x", "\n",paste(as.character(d6$study[3]),
                                                   paste("x (n =", as.character(d6$N[3]),")"))),
                              gsub("x", "\n",paste(as.character(d6$study[4]),
                                                   paste("x (n =", as.character(d6$N[4]),")"))))) +
  theme(panel.background = element_blank(),
        legend.position="none",
        axis.line.x = element_line(),
        axis.title.y = element_text(size=15,margin=margin(0,30,0,0)),
        axis.text.y = element_text(size=15),
        axis.title.x = element_text(size=15),
        axis.line.y = element_line(),
        plot.margin = unit(c(.1, 2,0,.5), "in"))
s3 <- ggdraw(s3)+
  draw_text(paste(formatC(d6$estimate[[4]], digits =  2, format = "f"),
                  "[",
                  formatC(d6$CI.lower[[4]],digits = 2, format = "f"),
                  ",", 
                  formatC(d6$CI.upper[[4]],digits = 2, format = "f"),
                  "]"), x = 0.75, y = 0.858, size = 12, hjust=0)+
  draw_text(paste(formatC(d6$estimate[[3]], digits = 2, format = "f"),
                  "[",
                  formatC(d6$CI.lower[[3]],digits = 2, format = "f"),
                  ",", 
                  formatC(d6$CI.upper[[3]],digits = 2, format = "f"),
                  "]"), x = 0.75, y = 0.6445, size = 12, hjust=0)+
  draw_text(paste(formatC(d6$estimate[[2]],digits = 2, format = "f"),
                  "[",
                  formatC(d6$CI.lower[[2]],digits = 2, format = "f"),
                  ",", 
                  formatC(d6$CI.upper[[2]],digits = 2, format = "f"),
                  "]"), x = 0.75, y =  0.432, size = 12, hjust=0)+
  draw_text(paste(formatC(d6$estimate[[1]],digits = 2, format = "f"),
                  "[",
                  formatC(d6$CI.lower[[1]],digits = 2, format = "f"),
                  ",", 
                  formatC(d6$CI.upper[[1]],digits = 2, format = "f"),
                  "]"), x = 0.75, y = 0.218, size = 12, hjust=0)

########################## Protocol 2 Meta Analysis #####################################

#re-order the levels for plotting
d7$study <- factor(as.character(d7$study), levels=desired_order)
d7 <- d7[order(d7$study),]

pro2 <- ggplot(data=d7,aes(x=estimate,y=d7$study))+
  geom_point(size=5, colour="black", fill = "black", shape = c(22,21,21,23)) +  
  geom_errorbarh(aes(xmin=CI.lower,xmax=CI.upper, height = .1)) +
  geom_vline(xintercept=0,linetype="dashed") +
  coord_cartesian(xlim=c(-1,8)) +
  scale_x_continuous(breaks= c(-1,0,1,2,3,4,5,6,7,8)) +
  ylab(d7$comparison[1]) +
  xlab("Cohen's"~italic("d")) +
  scale_y_discrete(labels = c(paste(as.character(d7$study[1])),
                              gsub("x", "\n",paste(as.character(d7$study[2]),
                                                   paste("x (n =", as.character(d7$N[2]),")"))),
                              gsub("x", "\n",paste(as.character(d7$study[3]),
                                                   paste("x (n =", as.character(d7$N[3]),")"))),
                              gsub("x", "\n",paste(as.character(d7$study[4]),
                                                   paste("x (n =", as.character(d7$N[4]),")"))))) +
  theme(panel.background = element_blank(),
        legend.position="none",
        axis.line.x = element_line(),
        axis.title.y = element_text(size=15,margin=margin(0,30,0,0)),
        axis.text.y = element_text(size=15),
        axis.line.y = element_line(),
        plot.margin = margin(.6, 8, .5, .55, "in"))
pro2 <- ggdraw(pro2)+
  draw_text(paste("Cohen's d","[","L.CI", ",", "U.CI", "]"), x = 0.568, y = 0.93, fontface="bold") +
  draw_text(paste(formatC(d7$estimate[[4]], digits =  2, format = "f"),
                  "[",
                  formatC(d7$CI.lower[[4]],digits = 2, format = "f"),
                  ",", 
                  formatC(d7$CI.upper[[4]],digits = 2, format = "f"),
                  "]"), x = 0.52, y = 0.79, size = 12, hjust=0)+
  draw_text(paste(formatC(d7$estimate[[3]], digits = 2, format = "f"),
                  "[",
                  formatC(d7$CI.lower[[3]],digits = 2, format = "f"),
                  ",", 
                  formatC(d7$CI.upper[[3]],digits = 2, format = "f"),
                  "]"), x = 0.52, y = 0.62, size = 12, hjust=0)+
  draw_text(paste(formatC(d7$estimate[[2]],digits = 2, format = "f"),
                  "[",
                  formatC(d7$CI.lower[[2]],digits = 2, format = "f"),
                  ",", 
                  formatC(d7$CI.upper[[2]],digits = 2, format = "f"),
                  "]"), x = 0.52, y =  0.455, size = 12, hjust=0)+
  draw_text(paste(formatC(d7$estimate[[1]],digits = 2, format = "f"),
                  "[",
                  formatC(d7$CI.lower[[1]],digits = 2, format = "f"),
                  ",", 
                  formatC(d7$CI.upper[[1]],digits = 2, format = "f"),
                  "]"), x = 0.52, y = 0.285, size = 12, hjust=0)


#Creates figure for protocol 3 meta-analyses
pro3 <- plot_grid(a1, s1, a2, s2, a3, s3, nrow = 3)

figure <- plot_grid(pro2, pro3, nrow = 2, rel_heights = c(1,3), labels = c("A", "B"), label_size = 25)
figure_3 <- plot_grid(figure,ncol = 1,rel_heights = c(0.1,1))

ggsave(file = "figure-3.png", plot = figure_3, width = 16, height = 24)
