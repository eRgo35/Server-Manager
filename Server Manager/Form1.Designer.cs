namespace Server_Manager
{
    partial class Main
    {
        /// <summary>
        /// Required designer variable.
        /// </summary>
        private System.ComponentModel.IContainer components = null;

        /// <summary>
        /// Clean up any resources being used.
        /// </summary>
        /// <param name="disposing">true if managed resources should be disposed; otherwise, false.</param>
        protected override void Dispose(bool disposing)
        {
            if (disposing && (components != null))
            {
                components.Dispose();
            }
            base.Dispose(disposing);
        }

        #region Windows Form Designer generated code

        /// <summary>
        /// Required method for Designer support - do not modify
        /// the contents of this method with the code editor.
        /// </summary>
        private void InitializeComponent()
        {
            this.components = new System.ComponentModel.Container();
            System.ComponentModel.ComponentResourceManager resources = new System.ComponentModel.ComponentResourceManager(typeof(Main));
            this.ping_button = new System.Windows.Forms.Button();
            this.power_on = new System.Windows.Forms.Button();
            this.power_off = new System.Windows.Forms.Button();
            this.Ping_Stuff = new System.Windows.Forms.GroupBox();
            this.label4 = new System.Windows.Forms.Label();
            this.label3 = new System.Windows.Forms.Label();
            this.label2 = new System.Windows.Forms.Label();
            this.label1 = new System.Windows.Forms.Label();
            this.Power = new System.Windows.Forms.GroupBox();
            this.Explore_Settings = new System.Windows.Forms.GroupBox();
            this.settings = new System.Windows.Forms.Button();
            this.open_win_exporer = new System.Windows.Forms.Button();
            this.timer1 = new System.Windows.Forms.Timer(this.components);
            this.Ping_Stuff.SuspendLayout();
            this.Power.SuspendLayout();
            this.Explore_Settings.SuspendLayout();
            this.SuspendLayout();
            // 
            // ping_button
            // 
            this.ping_button.Location = new System.Drawing.Point(6, 71);
            this.ping_button.Name = "ping_button";
            this.ping_button.Size = new System.Drawing.Size(259, 25);
            this.ping_button.TabIndex = 0;
            this.ping_button.Text = "Check Status";
            this.ping_button.UseVisualStyleBackColor = true;
            this.ping_button.Click += new System.EventHandler(this.Ping_button_Click);
            // 
            // power_on
            // 
            this.power_on.Enabled = false;
            this.power_on.Location = new System.Drawing.Point(5, 19);
            this.power_on.Name = "power_on";
            this.power_on.Size = new System.Drawing.Size(259, 25);
            this.power_on.TabIndex = 1;
            this.power_on.Text = "Power ON";
            this.power_on.UseVisualStyleBackColor = true;
            this.power_on.Click += new System.EventHandler(this.Power_on_Click);
            // 
            // power_off
            // 
            this.power_off.Enabled = false;
            this.power_off.Location = new System.Drawing.Point(5, 50);
            this.power_off.Name = "power_off";
            this.power_off.Size = new System.Drawing.Size(259, 25);
            this.power_off.TabIndex = 2;
            this.power_off.Text = "Power OFF";
            this.power_off.UseVisualStyleBackColor = true;
            this.power_off.Click += new System.EventHandler(this.Power_off_Click);
            // 
            // Ping_Stuff
            // 
            this.Ping_Stuff.Controls.Add(this.label4);
            this.Ping_Stuff.Controls.Add(this.label3);
            this.Ping_Stuff.Controls.Add(this.label2);
            this.Ping_Stuff.Controls.Add(this.label1);
            this.Ping_Stuff.Controls.Add(this.ping_button);
            this.Ping_Stuff.Location = new System.Drawing.Point(12, 12);
            this.Ping_Stuff.Name = "Ping_Stuff";
            this.Ping_Stuff.Size = new System.Drawing.Size(271, 105);
            this.Ping_Stuff.TabIndex = 3;
            this.Ping_Stuff.TabStop = false;
            this.Ping_Stuff.Text = "Server Status";
            // 
            // label4
            // 
            this.label4.AutoSize = true;
            this.label4.Location = new System.Drawing.Point(194, 44);
            this.label4.Name = "label4";
            this.label4.Size = new System.Drawing.Size(71, 13);
            this.label4.TabIndex = 4;
            this.label4.Text = "CHECKING...";
            // 
            // label3
            // 
            this.label3.AutoSize = true;
            this.label3.Location = new System.Drawing.Point(194, 20);
            this.label3.Name = "label3";
            this.label3.Size = new System.Drawing.Size(71, 13);
            this.label3.TabIndex = 3;
            this.label3.Text = "CHECKING...";
            // 
            // label2
            // 
            this.label2.AutoSize = true;
            this.label2.Location = new System.Drawing.Point(7, 44);
            this.label2.Name = "label2";
            this.label2.Size = new System.Drawing.Size(123, 13);
            this.label2.TabIndex = 2;
            this.label2.Text = "Operating System Status";
            // 
            // label1
            // 
            this.label1.AutoSize = true;
            this.label1.Location = new System.Drawing.Point(7, 20);
            this.label1.Name = "label1";
            this.label1.Size = new System.Drawing.Size(105, 13);
            this.label1.TabIndex = 1;
            this.label1.Text = "Network Card Status";
            // 
            // Power
            // 
            this.Power.Controls.Add(this.power_on);
            this.Power.Controls.Add(this.power_off);
            this.Power.Location = new System.Drawing.Point(13, 124);
            this.Power.Name = "Power";
            this.Power.Size = new System.Drawing.Size(270, 86);
            this.Power.TabIndex = 4;
            this.Power.TabStop = false;
            this.Power.Text = "Power";
            // 
            // Explore_Settings
            // 
            this.Explore_Settings.Controls.Add(this.settings);
            this.Explore_Settings.Controls.Add(this.open_win_exporer);
            this.Explore_Settings.Location = new System.Drawing.Point(13, 217);
            this.Explore_Settings.Name = "Explore_Settings";
            this.Explore_Settings.Size = new System.Drawing.Size(270, 85);
            this.Explore_Settings.TabIndex = 5;
            this.Explore_Settings.TabStop = false;
            this.Explore_Settings.Text = "Explore and Settings";
            // 
            // settings
            // 
            this.settings.Location = new System.Drawing.Point(9, 49);
            this.settings.Name = "settings";
            this.settings.Size = new System.Drawing.Size(255, 23);
            this.settings.TabIndex = 1;
            this.settings.Text = "Settings";
            this.settings.UseVisualStyleBackColor = true;
            this.settings.Click += new System.EventHandler(this.Settings_Click);
            // 
            // open_win_exporer
            // 
            this.open_win_exporer.Location = new System.Drawing.Point(9, 20);
            this.open_win_exporer.Name = "open_win_exporer";
            this.open_win_exporer.Size = new System.Drawing.Size(255, 23);
            this.open_win_exporer.TabIndex = 0;
            this.open_win_exporer.Text = "Open in Windows Explorer";
            this.open_win_exporer.UseVisualStyleBackColor = true;
            this.open_win_exporer.Click += new System.EventHandler(this.Open_win_exporer_Click);
            // 
            // timer1
            // 
            this.timer1.Interval = 8000;
            this.timer1.Tick += new System.EventHandler(this.Timer1_Tick);
            // 
            // Main
            // 
            this.AutoScaleDimensions = new System.Drawing.SizeF(6F, 13F);
            this.AutoScaleMode = System.Windows.Forms.AutoScaleMode.Font;
            this.ClientSize = new System.Drawing.Size(295, 314);
            this.Controls.Add(this.Explore_Settings);
            this.Controls.Add(this.Power);
            this.Controls.Add(this.Ping_Stuff);
            this.FormBorderStyle = System.Windows.Forms.FormBorderStyle.FixedSingle;
            this.Icon = ((System.Drawing.Icon)(resources.GetObject("$this.Icon")));
            this.MaximizeBox = false;
            this.Name = "Main";
            this.StartPosition = System.Windows.Forms.FormStartPosition.CenterScreen;
            this.Text = "Server Power Manager";
            this.Load += new System.EventHandler(this.Main_Load);
            this.Ping_Stuff.ResumeLayout(false);
            this.Ping_Stuff.PerformLayout();
            this.Power.ResumeLayout(false);
            this.Explore_Settings.ResumeLayout(false);
            this.ResumeLayout(false);

        }

        #endregion

        private System.Windows.Forms.Button ping_button;
        private System.Windows.Forms.Button power_on;
        private System.Windows.Forms.Button power_off;
        private System.Windows.Forms.GroupBox Ping_Stuff;
        private System.Windows.Forms.Label label4;
        private System.Windows.Forms.Label label3;
        private System.Windows.Forms.Label label2;
        private System.Windows.Forms.Label label1;
        private System.Windows.Forms.GroupBox Power;
        private System.Windows.Forms.GroupBox Explore_Settings;
        private System.Windows.Forms.Button settings;
        private System.Windows.Forms.Button open_win_exporer;
        private System.Windows.Forms.Timer timer1;
    }
}

