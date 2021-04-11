<Global.Microsoft.VisualBasic.CompilerServices.DesignerGenerated()>
Partial Class Main
    Inherits System.Windows.Forms.Form

    'Form overrides dispose to clean up the component list.
    <System.Diagnostics.DebuggerNonUserCode()>
    Protected Overrides Sub Dispose(ByVal disposing As Boolean)
        Try
            If disposing AndAlso components IsNot Nothing Then
                components.Dispose()
            End If
        Finally
            MyBase.Dispose(disposing)
        End Try
    End Sub

    'Required by the Windows Form Designer
    Private components As System.ComponentModel.IContainer

    'NOTE: The following procedure is required by the Windows Form Designer
    'It can be modified using the Windows Form Designer.  
    'Do not modify it using the code editor.
    <System.Diagnostics.DebuggerStepThrough()>
    Private Sub InitializeComponent()
        Me.components = New System.ComponentModel.Container()
        Dim resources As System.ComponentModel.ComponentResourceManager = New System.ComponentModel.ComponentResourceManager(GetType(Main))
        Me.STATUS_POWER = New System.Windows.Forms.Label()
        Me.STATUS_SERVER = New System.Windows.Forms.Label()
        Me.STATUS1 = New System.Windows.Forms.Label()
        Me.STATUS2 = New System.Windows.Forms.Label()
        Me.GROUPBOX = New System.Windows.Forms.GroupBox()
        Me.AVABILITY = New System.Windows.Forms.Button()
        Me.POWER_ON_BUTTON = New System.Windows.Forms.Button()
        Me.POWER_OFF_BUTTON = New System.Windows.Forms.Button()
        Me.Timer = New System.Windows.Forms.Timer(Me.components)
        Me.SettingWindow = New System.Windows.Forms.Button()
        Me.STATUS3 = New System.Windows.Forms.Label()
        Me.STATUS4 = New System.Windows.Forms.Label()
        Me.ping2 = New System.Windows.Forms.Button()
        Me.ping1 = New System.Windows.Forms.Button()
        Me.Mod2 = New System.Windows.Forms.CheckBox()
        Me.Mod1 = New System.Windows.Forms.CheckBox()
        Me.OS_IP = New System.Windows.Forms.TextBox()
        Me.POWER_IP = New System.Windows.Forms.TextBox()
        Me.Label2 = New System.Windows.Forms.Label()
        Me.Label1 = New System.Windows.Forms.Label()
        Me.Save = New System.Windows.Forms.Button()
        Me.GroupBox1 = New System.Windows.Forms.GroupBox()
        Me.GROUPBOX.SuspendLayout()
        Me.GroupBox1.SuspendLayout()
        Me.SuspendLayout()
        '
        'STATUS_POWER
        '
        Me.STATUS_POWER.AutoSize = True
        Me.STATUS_POWER.Location = New System.Drawing.Point(6, 22)
        Me.STATUS_POWER.Name = "STATUS_POWER"
        Me.STATUS_POWER.Size = New System.Drawing.Size(70, 13)
        Me.STATUS_POWER.TabIndex = 0
        Me.STATUS_POWER.Text = "Power Status"
        '
        'STATUS_SERVER
        '
        Me.STATUS_SERVER.AutoSize = True
        Me.STATUS_SERVER.Location = New System.Drawing.Point(6, 48)
        Me.STATUS_SERVER.Name = "STATUS_SERVER"
        Me.STATUS_SERVER.Size = New System.Drawing.Size(55, 13)
        Me.STATUS_SERVER.TabIndex = 1
        Me.STATUS_SERVER.Text = "OS Status"
        '
        'STATUS1
        '
        Me.STATUS1.AutoSize = True
        Me.STATUS1.Location = New System.Drawing.Point(184, 22)
        Me.STATUS1.Name = "STATUS1"
        Me.STATUS1.Size = New System.Drawing.Size(50, 13)
        Me.STATUS1.TabIndex = 2
        Me.STATUS1.Text = "STATUS"
        '
        'STATUS2
        '
        Me.STATUS2.AutoSize = True
        Me.STATUS2.Location = New System.Drawing.Point(184, 48)
        Me.STATUS2.Name = "STATUS2"
        Me.STATUS2.Size = New System.Drawing.Size(50, 13)
        Me.STATUS2.TabIndex = 3
        Me.STATUS2.Text = "STATUS"
        '
        'GROUPBOX
        '
        Me.GROUPBOX.Controls.Add(Me.STATUS2)
        Me.GROUPBOX.Controls.Add(Me.STATUS_POWER)
        Me.GROUPBOX.Controls.Add(Me.STATUS1)
        Me.GROUPBOX.Controls.Add(Me.STATUS_SERVER)
        Me.GROUPBOX.Location = New System.Drawing.Point(12, 99)
        Me.GROUPBOX.Name = "GROUPBOX"
        Me.GROUPBOX.Size = New System.Drawing.Size(250, 75)
        Me.GROUPBOX.TabIndex = 4
        Me.GROUPBOX.TabStop = False
        Me.GROUPBOX.Text = "Power Managment"
        '
        'AVABILITY
        '
        Me.AVABILITY.Location = New System.Drawing.Point(12, 12)
        Me.AVABILITY.Name = "AVABILITY"
        Me.AVABILITY.Size = New System.Drawing.Size(221, 23)
        Me.AVABILITY.TabIndex = 5
        Me.AVABILITY.Text = "Refresh"
        Me.AVABILITY.UseVisualStyleBackColor = True
        '
        'POWER_ON_BUTTON
        '
        Me.POWER_ON_BUTTON.Enabled = False
        Me.POWER_ON_BUTTON.Location = New System.Drawing.Point(12, 41)
        Me.POWER_ON_BUTTON.Name = "POWER_ON_BUTTON"
        Me.POWER_ON_BUTTON.Size = New System.Drawing.Size(250, 23)
        Me.POWER_ON_BUTTON.TabIndex = 6
        Me.POWER_ON_BUTTON.Text = "Enable Server"
        Me.POWER_ON_BUTTON.UseVisualStyleBackColor = True
        '
        'POWER_OFF_BUTTON
        '
        Me.POWER_OFF_BUTTON.Enabled = False
        Me.POWER_OFF_BUTTON.Location = New System.Drawing.Point(12, 70)
        Me.POWER_OFF_BUTTON.Name = "POWER_OFF_BUTTON"
        Me.POWER_OFF_BUTTON.Size = New System.Drawing.Size(250, 23)
        Me.POWER_OFF_BUTTON.TabIndex = 7
        Me.POWER_OFF_BUTTON.Text = "Disable Server"
        Me.POWER_OFF_BUTTON.UseVisualStyleBackColor = True
        '
        'Timer
        '
        Me.Timer.Enabled = True
        Me.Timer.Interval = 3000
        '
        'SettingWindow
        '
        Me.SettingWindow.Location = New System.Drawing.Point(239, 12)
        Me.SettingWindow.Name = "SettingWindow"
        Me.SettingWindow.Size = New System.Drawing.Size(23, 23)
        Me.SettingWindow.TabIndex = 8
        Me.SettingWindow.Text = ">"
        Me.SettingWindow.UseVisualStyleBackColor = True
        '
        'STATUS3
        '
        Me.STATUS3.AutoSize = True
        Me.STATUS3.Location = New System.Drawing.Point(161, 41)
        Me.STATUS3.Name = "STATUS3"
        Me.STATUS3.Size = New System.Drawing.Size(50, 13)
        Me.STATUS3.TabIndex = 23
        Me.STATUS3.Text = "STATUS"
        '
        'STATUS4
        '
        Me.STATUS4.AutoSize = True
        Me.STATUS4.Location = New System.Drawing.Point(161, 92)
        Me.STATUS4.Name = "STATUS4"
        Me.STATUS4.Size = New System.Drawing.Size(50, 13)
        Me.STATUS4.TabIndex = 22
        Me.STATUS4.Text = "STATUS"
        '
        'ping2
        '
        Me.ping2.Enabled = False
        Me.ping2.Location = New System.Drawing.Point(80, 87)
        Me.ping2.Name = "ping2"
        Me.ping2.Size = New System.Drawing.Size(75, 23)
        Me.ping2.TabIndex = 21
        Me.ping2.Text = "Ping"
        Me.ping2.UseVisualStyleBackColor = True
        '
        'ping1
        '
        Me.ping1.Enabled = False
        Me.ping1.Location = New System.Drawing.Point(80, 36)
        Me.ping1.Name = "ping1"
        Me.ping1.Size = New System.Drawing.Size(75, 23)
        Me.ping1.TabIndex = 20
        Me.ping1.Text = "Ping"
        Me.ping1.UseVisualStyleBackColor = True
        '
        'Mod2
        '
        Me.Mod2.AutoSize = True
        Me.Mod2.Location = New System.Drawing.Point(186, 67)
        Me.Mod2.Name = "Mod2"
        Me.Mod2.Size = New System.Drawing.Size(57, 17)
        Me.Mod2.TabIndex = 19
        Me.Mod2.Text = "Modify"
        Me.Mod2.UseVisualStyleBackColor = True
        '
        'Mod1
        '
        Me.Mod1.AutoSize = True
        Me.Mod1.Location = New System.Drawing.Point(186, 16)
        Me.Mod1.Name = "Mod1"
        Me.Mod1.Size = New System.Drawing.Size(57, 17)
        Me.Mod1.TabIndex = 18
        Me.Mod1.Text = "Modify"
        Me.Mod1.UseVisualStyleBackColor = True
        '
        'OS_IP
        '
        Me.OS_IP.Enabled = False
        Me.OS_IP.Location = New System.Drawing.Point(80, 65)
        Me.OS_IP.Name = "OS_IP"
        Me.OS_IP.Size = New System.Drawing.Size(100, 20)
        Me.OS_IP.TabIndex = 17
        Me.OS_IP.Text = "0.0.0.0"
        '
        'POWER_IP
        '
        Me.POWER_IP.Enabled = False
        Me.POWER_IP.Location = New System.Drawing.Point(80, 14)
        Me.POWER_IP.Name = "POWER_IP"
        Me.POWER_IP.Size = New System.Drawing.Size(100, 20)
        Me.POWER_IP.TabIndex = 16
        Me.POWER_IP.Text = "0.0.0.0"
        '
        'Label2
        '
        Me.Label2.AutoSize = True
        Me.Label2.Location = New System.Drawing.Point(9, 74)
        Me.Label2.Name = "Label2"
        Me.Label2.Size = New System.Drawing.Size(35, 13)
        Me.Label2.TabIndex = 15
        Me.Label2.Text = "OS IP"
        '
        'Label1
        '
        Me.Label1.AutoSize = True
        Me.Label1.Location = New System.Drawing.Point(9, 23)
        Me.Label1.Name = "Label1"
        Me.Label1.Size = New System.Drawing.Size(51, 13)
        Me.Label1.TabIndex = 14
        Me.Label1.Text = "Server IP"
        '
        'Save
        '
        Me.Save.Location = New System.Drawing.Point(12, 130)
        Me.Save.Name = "Save"
        Me.Save.Size = New System.Drawing.Size(234, 24)
        Me.Save.TabIndex = 24
        Me.Save.Text = "Save"
        Me.Save.UseVisualStyleBackColor = True
        '
        'GroupBox1
        '
        Me.GroupBox1.Controls.Add(Me.Save)
        Me.GroupBox1.Controls.Add(Me.Label1)
        Me.GroupBox1.Controls.Add(Me.STATUS3)
        Me.GroupBox1.Controls.Add(Me.Label2)
        Me.GroupBox1.Controls.Add(Me.STATUS4)
        Me.GroupBox1.Controls.Add(Me.POWER_IP)
        Me.GroupBox1.Controls.Add(Me.ping2)
        Me.GroupBox1.Controls.Add(Me.OS_IP)
        Me.GroupBox1.Controls.Add(Me.ping1)
        Me.GroupBox1.Controls.Add(Me.Mod1)
        Me.GroupBox1.Controls.Add(Me.Mod2)
        Me.GroupBox1.Location = New System.Drawing.Point(270, 12)
        Me.GroupBox1.Name = "GroupBox1"
        Me.GroupBox1.Size = New System.Drawing.Size(252, 162)
        Me.GroupBox1.TabIndex = 25
        Me.GroupBox1.TabStop = False
        Me.GroupBox1.Text = "Settings"
        '
        'Main
        '
        Me.AutoScaleDimensions = New System.Drawing.SizeF(6.0!, 13.0!)
        Me.AutoScaleMode = System.Windows.Forms.AutoScaleMode.Font
        Me.ClientSize = New System.Drawing.Size(534, 183)
        Me.Controls.Add(Me.GroupBox1)
        Me.Controls.Add(Me.SettingWindow)
        Me.Controls.Add(Me.POWER_OFF_BUTTON)
        Me.Controls.Add(Me.POWER_ON_BUTTON)
        Me.Controls.Add(Me.AVABILITY)
        Me.Controls.Add(Me.GROUPBOX)
        Me.FormBorderStyle = System.Windows.Forms.FormBorderStyle.FixedSingle
        Me.Icon = CType(resources.GetObject("$this.Icon"), System.Drawing.Icon)
        Me.MaximizeBox = False
        Me.Name = "Main"
        Me.Text = "Disk Manager"
        Me.GROUPBOX.ResumeLayout(False)
        Me.GROUPBOX.PerformLayout()
        Me.GroupBox1.ResumeLayout(False)
        Me.GroupBox1.PerformLayout()
        Me.ResumeLayout(False)

    End Sub

    Friend WithEvents STATUS_POWER As Label
    Friend WithEvents STATUS_SERVER As Label
    Friend WithEvents STATUS1 As Label
    Friend WithEvents STATUS2 As Label
    Friend WithEvents GROUPBOX As GroupBox
    Friend WithEvents AVABILITY As Button
    Friend WithEvents POWER_ON_BUTTON As Button
    Friend WithEvents POWER_OFF_BUTTON As Button
    Friend WithEvents Timer As Timer
    Friend WithEvents SettingWindow As Button
    Friend WithEvents STATUS3 As Label
    Friend WithEvents STATUS4 As Label
    Friend WithEvents ping2 As Button
    Friend WithEvents ping1 As Button
    Friend WithEvents Mod2 As CheckBox
    Friend WithEvents Mod1 As CheckBox
    Friend WithEvents OS_IP As TextBox
    Friend WithEvents POWER_IP As TextBox
    Friend WithEvents Label2 As Label
    Friend WithEvents Label1 As Label
    Friend WithEvents Save As Button
    Friend WithEvents GroupBox1 As GroupBox
End Class
