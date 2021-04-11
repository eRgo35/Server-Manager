Imports System.Net.NetworkInformation
Imports VB = Microsoft.VisualBasic
Public Class Main
    Private Sub wait(ByVal seconds As Integer)
        For i As Integer = 0 To seconds * 100
            System.Threading.Thread.Sleep(10)
            Application.DoEvents()
        Next
    End Sub
    Private Sub AVABILITY_Click(sender As Object, e As EventArgs) Handles AVABILITY.Click
        sender.Enabled = False
        STATUS1.Text = "Refreshing..."
        STATUS2.Text = "Refreshing..."
        STATUS1.ForeColor = Color.Black
        STATUS2.ForeColor = Color.Black

        wait(1)

        If My.Computer.Network.Ping(POWER_IP.Text) Then
            STATUS1.Text = "ONLINE"
            STATUS1.ForeColor = Color.Green
        Else
            STATUS1.Text = "OFFLINE"
            STATUS1.ForeColor = Color.Red
        End If
        If My.Computer.Network.Ping(OS_IP.Text) Then
            STATUS2.Text = "ONLINE"
            STATUS2.ForeColor = Color.Green
        Else
            STATUS2.Text = "OFFLINE"
            STATUS2.ForeColor = Color.Red
        End If
        If STATUS1.ForeColor = Color.Red And STATUS2.ForeColor = Color.Red Then
            POWER_OFF_BUTTON.Enabled = False
            POWER_ON_BUTTON.Enabled = False
        ElseIf STATUS1.ForeColor = Color.Green And STATUS2.ForeColor = Color.Red
            POWER_OFF_BUTTON.Enabled = False
            POWER_ON_BUTTON.Enabled = True
        ElseIf STATUS1.ForeColor = Color.Green And STATUS2.ForeColor = Color.Green
            POWER_OFF_BUTTON.Enabled = True
            POWER_ON_BUTTON.Enabled = False
        End If
        sender.Enabled = True
    End Sub
    Private Sub Main_Load(sender As Object, e As EventArgs) Handles MyBase.Load
        Me.Size = New Size(289, 222)
        POWER_IP.Text = My.Settings.POWER_IP
        OS_IP.Text = My.Settings.OS_IP

        If My.Computer.Network.Ping(POWER_IP.Text) Then
            STATUS1.Text = "ONLINE"
            STATUS1.ForeColor = Color.Green
        Else
            STATUS1.Text = "OFFLINE"
            STATUS1.ForeColor = Color.Red
        End If
        If My.Computer.Network.Ping(OS_IP.Text) Then
            STATUS2.Text = "ONLINE"
            STATUS2.ForeColor = Color.Green
        Else
            STATUS2.Text = "OFFLINE"
            STATUS2.ForeColor = Color.Red
        End If
        If STATUS1.ForeColor = Color.Red And STATUS2.ForeColor = Color.Red Then
            POWER_OFF_BUTTON.Enabled = False
            POWER_ON_BUTTON.Enabled = False
        ElseIf STATUS1.ForeColor = Color.Green And STATUS2.ForeColor = Color.Red
            POWER_OFF_BUTTON.Enabled = False
            POWER_ON_BUTTON.Enabled = True
        ElseIf STATUS1.ForeColor = Color.Green And STATUS2.ForeColor = Color.Green
            POWER_OFF_BUTTON.Enabled = True
            POWER_ON_BUTTON.Enabled = False
        End If
    End Sub
    Private Sub SettingWindow_Click(sender As Object, e As EventArgs) Handles SettingWindow.Click
        If SettingWindow.Text = ">" Then
            Me.Size = New Size(550, 222)
            sender.text = "<"
        ElseIf SettingWindow.Text = "<"
            Me.Size = New Size(289, 222)
            sender.text = ">"
        End If
    End Sub

    Private Sub Save_Click(sender As Object, e As EventArgs) Handles Save.Click
        My.Settings.POWER_IP = POWER_IP.Text
        My.Settings.OS_IP = OS_IP.Text
        My.Settings.Save()
        sender.text = "Save Successful! Refreshing IPs.."
        sender.Enabled = False
        AVABILITY.Enabled = False
        STATUS1.Text = "Refreshing..."
        STATUS2.Text = "Refreshing..."
        STATUS1.ForeColor = Color.Black
        STATUS2.ForeColor = Color.Black

        wait(1)

        If My.Computer.Network.Ping(POWER_IP.Text) Then
            STATUS1.Text = "ONLINE"
            STATUS1.ForeColor = Color.Green
        Else
            STATUS1.Text = "OFFLINE"
            STATUS1.ForeColor = Color.Red
        End If
        If My.Computer.Network.Ping(OS_IP.Text) Then
            STATUS2.Text = "ONLINE"
            STATUS2.ForeColor = Color.Green
        Else
            STATUS2.Text = "OFFLINE"
            STATUS2.ForeColor = Color.Red
        End If
        If STATUS1.ForeColor = Color.Red And STATUS2.ForeColor = Color.Red Then
            POWER_OFF_BUTTON.Enabled = False
            POWER_ON_BUTTON.Enabled = False
        ElseIf STATUS1.ForeColor = Color.Green And STATUS2.ForeColor = Color.Red
            POWER_OFF_BUTTON.Enabled = False
            POWER_ON_BUTTON.Enabled = True
        ElseIf STATUS1.ForeColor = Color.Green And STATUS2.ForeColor = Color.Green
            POWER_OFF_BUTTON.Enabled = True
            POWER_ON_BUTTON.Enabled = False
        End If
        sender.Enabled = True
        sender.text = "Save"
        AVABILITY.Enabled = True
    End Sub

    Private Sub Mod1_CheckedChanged(sender As Object, e As EventArgs) Handles Mod1.CheckedChanged
        If POWER_IP.Enabled = False Then
            POWER_IP.Enabled = True
            ping1.Enabled = True
        ElseIf POWER_IP.Enabled = True Then
            POWER_IP.Enabled = False
            ping1.Enabled = False
        End If
    End Sub
    Private Sub Mod2_CheckedChanged(sender As Object, e As EventArgs) Handles Mod2.CheckedChanged
        If OS_IP.Enabled = False Then
            OS_IP.Enabled = True
            ping2.Enabled = True
        ElseIf OS_IP.Enabled = True Then
            OS_IP.Enabled = False
            ping2.Enabled = False
        End If
    End Sub

    Private Sub ping1_Click(sender As Object, e As EventArgs) Handles ping1.Click
        sender.Enabled = False
        AVABILITY.Enabled = False
        STATUS3.Text = "Refreshing..."
        STATUS3.ForeColor = Color.Black
        wait(1)
        If My.Computer.Network.Ping(POWER_IP.Text) Then
            STATUS3.Text = "ONLINE"
            STATUS3.ForeColor = Color.Green
        Else
            STATUS3.Text = "OFFLINE"
            STATUS3.ForeColor = Color.Red
        End If
        sender.Enabled = True
        AVABILITY.Enabled = True
    End Sub

    Private Sub ping2_Click(sender As Object, e As EventArgs) Handles ping2.Click
        sender.Enabled = False
        AVABILITY.Enabled = False
        STATUS4.Text = "Refreshing..."
        STATUS4.ForeColor = Color.Black
        wait(1)
        If My.Computer.Network.Ping(OS_IP.Text) Then
            STATUS4.Text = "ONLINE"
            STATUS4.ForeColor = Color.Green
        Else
            STATUS4.Text = "OFFLINE"
            STATUS4.ForeColor = Color.Red
        End If
        AVABILITY.Enabled = True
        sender.Enabled = True
    End Sub

    Private Sub POWER_ON_BUTTON_Click(sender As Object, e As EventArgs) Handles POWER_ON_BUTTON.Click
        MsgBox("To be implemented..")
    End Sub

    Private Sub POWER_OFF_BUTTON_Click(sender As Object, e As EventArgs) Handles POWER_OFF_BUTTON.Click
        MsgBox("To be implemented..")
    End Sub
End Class
