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
            this.Ping_Button = new System.Windows.Forms.Button();
            this.Power_ON = new System.Windows.Forms.Button();
            this.Power_OFF = new System.Windows.Forms.Button();
            this.Ping_Stuff = new System.Windows.Forms.GroupBox();
            this.label3 = new System.Windows.Forms.Label();
            this.label1 = new System.Windows.Forms.Label();
            this.Power = new System.Windows.Forms.GroupBox();
            this.Explore_Settings = new System.Windows.Forms.GroupBox();
            this.Settings = new System.Windows.Forms.Button();
            this.Open_Win_Explorer = new System.Windows.Forms.Button();
            this.Auto_Ping = new System.Windows.Forms.Timer(this.components);
            this.Ping_Stuff.SuspendLayout();
            this.Power.SuspendLayout();
            this.Explore_Settings.SuspendLayout();
            this.SuspendLayout();
            // 
            // Ping_Button
            // 
            this.Ping_Button.Location = new System.Drawing.Point(6, 36);
            this.Ping_Button.Name = "Ping_Button";
            this.Ping_Button.Size = new System.Drawing.Size(259, 25);
            this.Ping_Button.TabIndex = 0;
            this.Ping_Button.Text = "Check Status";
            this.Ping_Button.UseVisualStyleBackColor = true;
            this.Ping_Button.Click += new System.EventHandler(this.Ping_button_Click);
            // 
            // Power_ON
            // 
            this.Power_ON.Enabled = false;
            this.Power_ON.Location = new System.Drawing.Point(5, 19);
            this.Power_ON.Name = "Power_ON";
            this.Power_ON.Size = new System.Drawing.Size(259, 25);
            this.Power_ON.TabIndex = 1;
            this.Power_ON.Text = "Power ON";
            this.Power_ON.UseVisualStyleBackColor = true;
            this.Power_ON.Click += new System.EventHandler(this.Power_on_Click);
            // 
            // Power_OFF
            // 
            this.Power_OFF.Enabled = false;
            this.Power_OFF.Location = new System.Drawing.Point(5, 50);
            this.Power_OFF.Name = "Power_OFF";
            this.Power_OFF.Size = new System.Drawing.Size(259, 25);
            this.Power_OFF.TabIndex = 2;
            this.Power_OFF.Text = "Power OFF";
            this.Power_OFF.UseVisualStyleBackColor = true;
            this.Power_OFF.Click += new System.EventHandler(this.Power_off_Click);
            // 
            // Ping_Stuff
            // 
            this.Ping_Stuff.Controls.Add(this.label3);
            this.Ping_Stuff.Controls.Add(this.label1);
            this.Ping_Stuff.Controls.Add(this.Ping_Button);
            this.Ping_Stuff.Location = new System.Drawing.Point(12, 12);
            this.Ping_Stuff.Name = "Ping_Stuff";
            this.Ping_Stuff.Size = new System.Drawing.Size(271, 74);
            this.Ping_Stuff.TabIndex = 3;
            this.Ping_Stuff.TabStop = false;
            this.Ping_Stuff.Text = "Server Status";
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
            this.Power.Controls.Add(this.Power_ON);
            this.Power.Controls.Add(this.Power_OFF);
            this.Power.Location = new System.Drawing.Point(13, 92);
            this.Power.Name = "Power";
            this.Power.Size = new System.Drawing.Size(270, 86);
            this.Power.TabIndex = 4;
            this.Power.TabStop = false;
            this.Power.Text = "Power";
            // 
            // Explore_Settings
            // 
            this.Explore_Settings.Controls.Add(this.Settings);
            this.Explore_Settings.Controls.Add(this.Open_Win_Explorer);
            this.Explore_Settings.Location = new System.Drawing.Point(13, 184);
            this.Explore_Settings.Name = "Explore_Settings";
            this.Explore_Settings.Size = new System.Drawing.Size(270, 85);
            this.Explore_Settings.TabIndex = 5;
            this.Explore_Settings.TabStop = false;
            this.Explore_Settings.Text = "Explore and Settings";
            // 
            // Settings
            // 
            this.Settings.Location = new System.Drawing.Point(9, 49);
            this.Settings.Name = "Settings";
            this.Settings.Size = new System.Drawing.Size(255, 23);
            this.Settings.TabIndex = 1;
            this.Settings.Text = "Settings";
            this.Settings.UseVisualStyleBackColor = true;
            this.Settings.Click += new System.EventHandler(this.Settings_Click);
            // 
            // Open_Win_Explorer
            // 
            this.Open_Win_Explorer.Location = new System.Drawing.Point(9, 20);
            this.Open_Win_Explorer.Name = "Open_Win_Explorer";
            this.Open_Win_Explorer.Size = new System.Drawing.Size(255, 23);
            this.Open_Win_Explorer.TabIndex = 0;
            this.Open_Win_Explorer.Text = "Open in Windows Explorer";
            this.Open_Win_Explorer.UseVisualStyleBackColor = true;
            this.Open_Win_Explorer.Click += new System.EventHandler(this.Open_win_exporer_Click);
            // 
            // Auto_Ping
            // 
            this.Auto_Ping.Interval = 8000;
            this.Auto_Ping.Tick += new System.EventHandler(this.Auto_Ping_Tick);
            // 
            // Main
            // 
            this.AutoScaleDimensions = new System.Drawing.SizeF(6F, 13F);
            this.AutoScaleMode = System.Windows.Forms.AutoScaleMode.Font;
            this.ClientSize = new System.Drawing.Size(295, 282);
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

        private System.Windows.Forms.Button Ping_Button;
        private System.Windows.Forms.Button Power_ON;
        private System.Windows.Forms.Button Power_OFF;
        private System.Windows.Forms.GroupBox Ping_Stuff;
        private System.Windows.Forms.Label label3;
        private System.Windows.Forms.Label label1;
        private System.Windows.Forms.GroupBox Power;
        private System.Windows.Forms.GroupBox Explore_Settings;
        private System.Windows.Forms.Button Settings;
        private System.Windows.Forms.Button Open_Win_Explorer;
        private System.Windows.Forms.Timer Auto_Ping;
    }
}

